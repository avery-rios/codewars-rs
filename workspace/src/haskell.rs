use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use super::{Code, WorkspaceObject};

const STATE_FILE: &str = "haskell_state.json";

/// get haskell module name
/// requires no comment before module keyword
fn module_name(src: &str) -> Option<&str> {
    Some(
        src.trim_start()
            .strip_prefix("module")?
            .trim_start()
            .split_once(char::is_whitespace)?
            .0,
    )
}

#[derive(Serialize, Deserialize)]
struct State<'a> {
    code_path: &'a str,
    fixture_path: &'a str,
}

#[derive(Debug, thiserror::Error)]
enum OpenErrInner {
    #[error("failed to open state")]
    Io(#[source] io::Error),
    #[error("failed to deserialize json")]
    Json(#[source] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct OpenError(#[from] OpenErrInner);

#[derive(Debug, thiserror::Error)]
enum CreateErrorInner {
    #[error("unknown code {0}")]
    UnknownCode(String),
    #[error("unknown test {0}")]
    UnknownTest(String),
    #[error("failed to write file")]
    Write(#[source] io::Error),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct CreateError(#[from] CreateErrorInner);

pub struct Haskell {
    root: PathBuf,
    code_path: PathBuf,
    fixture_path: PathBuf,
}

impl Haskell {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, OpenError> {
        let mut root = root.as_ref().to_path_buf();

        root.push(STATE_FILE);
        let stat_str = fs::read(&root).map_err(OpenErrInner::Io)?;
        root.pop();

        let stat: State<'_> = serde_json::from_slice(&stat_str).map_err(OpenErrInner::Json)?;
        Ok(Self {
            code_path: root.join(stat.code_path),
            fixture_path: root.join(stat.fixture_path),
            root,
        })
    }
    pub fn create(root: impl AsRef<Path>, code: &str, test: &str) -> Result<Self, CreateError> {
        fn write_code(
            root: &Path,
            base: &str,
            module: &str,
            code: &str,
        ) -> Result<(String, PathBuf), io::Error> {
            let mut rel_path = String::new();
            rel_path.push_str(base);
            for m in module.split('.') {
                rel_path.push('/');
                rel_path.push_str(m);
            }
            rel_path.push_str(".hs");

            let path = root.join(&rel_path);
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(&path, code)?;
            Ok((rel_path, path))
        }
        let root = root.as_ref();
        let mut path = root.to_path_buf();

        let code_mod =
            module_name(code).ok_or_else(|| CreateErrorInner::UnknownCode(code.to_string()))?;
        let (code_path_rel, code_path) =
            write_code(root, "src", code_mod, code).map_err(CreateErrorInner::Write)?;

        let test_mod =
            module_name(test).ok_or_else(|| CreateErrorInner::UnknownTest(test.to_string()))?;
        let (test_path_rel, test_path) =
            write_code(root, "test/sample", test_mod, test).map_err(CreateErrorInner::Write)?;
        {
            path.push("test/sample/Main.hs");
            fs::write(
                &path,
                format!(
                    "module Main (main) where\n\
                     \n\
                     import Test.Hspec\n\
                     import {} (spec)\n\
                     \n\
                     main :: IO ()\n\
                     main = hspec spec",
                    test_mod
                ),
            )
            .map_err(CreateErrorInner::Write)?;
            path.pop();
            path.pop();
            path.pop();
        }

        {
            path.push(STATE_FILE);
            fs::write(
                &path,
                serde_json::to_vec(&State {
                    code_path: &code_path_rel,
                    fixture_path: &test_path_rel,
                })
                .unwrap(),
            )
            .map_err(CreateErrorInner::Write)?;
            path.pop();
        }

        {
            path.push("challenge.cabal");
            fs::write(
                &path,
                format!(
                    include_str!("./haskell/challenge.cabal"),
                    code_module = code_mod,
                    test_module = test_mod
                ),
            )
            .map_err(CreateErrorInner::Write)?;
            path.pop();
        }

        {
            path.push("cabal.project.local");
            fs::write(&path, include_str!("./haskell/cabal.project.local.in"))
                .map_err(CreateErrorInner::Write)?;
        }

        Ok(Self {
            root: root.to_path_buf(),
            code_path,
            fixture_path: test_path,
        })
    }
}
impl WorkspaceObject for Haskell {
    fn root(&self) -> &Path {
        self.root.as_path()
    }
    fn get_code(&self) -> Result<Code, io::Error> {
        Ok(Code {
            solution: fs::read_to_string(&self.code_path)?,
            fixture: fs::read_to_string(&self.fixture_path)?,
        })
    }
    fn clean(&self) -> Result<(), io::Error> {
        let mut path = self.root.clone();

        path.push(STATE_FILE);
        if path.exists() {
            fs::remove_file(&path)?;
        }
        path.pop();

        path.push("cabal.project.local");
        if path.exists() {
            fs::remove_file(&path)?;
        }
        path.pop();

        path.push("cabal.project.local~");
        if path.exists() {
            fs::remove_file(&path)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    mod mod_name {
        use super::super::module_name;

        #[test]
        fn simple() {
            assert_eq!(module_name("module M1.M2.M3 where").unwrap(), "M1.M2.M3");
            assert_eq!(
                module_name("module M1.M2.M3 (f1, f2, f3) where").unwrap(),
                "M1.M2.M3"
            );
        }

        #[test]
        fn multiline_name() {
            assert_eq!(
                module_name("module M1.M2\n    ( f1, f2, f3) where").unwrap(),
                "M1.M2"
            );
        }
    }
}
