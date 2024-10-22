use std::{
    ffi::{CStr, CString},
    io,
    path::Path,
};

use rustix::{
    fd::{AsFd, BorrowedFd, OwnedFd},
    io::Errno,
};
use serde::{Deserialize, Serialize};

use crate::{
    util::{call_command_at, fs},
    Code, Config, WorkspaceObject,
};

const STATE_FILE: &CStr = c"haskell_state.json";

/// get haskell module name
fn module_name(src: &str) -> Option<&str> {
    let mut s = src.trim_start();
    while s.starts_with("{-") {
        let (_, t) = s.split_once("-}")?;
        s = t.trim_start();
    }
    while s.starts_with("-- ") {
        let (_, t) = s.split_once('\n')?;
        s = t.trim_start();
    }
    Some(
        s.strip_prefix("module")?
            .trim_start()
            .split_once(char::is_whitespace)?
            .0,
    )
}

#[derive(Serialize, Deserialize)]
struct State {
    code_path: CString,
    fixture_path: CString,
}

#[derive(Debug, thiserror::Error)]
enum OpenErrInner {
    #[error("failed to open state")]
    Io(#[source] Errno),
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
    #[error("failed to write source code")]
    WriteCode(#[source] io::Error),
    #[error("io error")]
    Io(#[source] Errno),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct CreateError(#[from] CreateErrorInner);

pub struct Haskell {
    root: OwnedFd,
    state: State,
}

impl Haskell {
    pub fn open(root: &Path) -> Result<Self, OpenError> {
        let root = fs::open_dirfd(root).map_err(OpenErrInner::Io)?;
        Ok(Self {
            state: serde_json::from_slice(
                &fs::read(root.as_fd(), STATE_FILE).map_err(OpenErrInner::Io)?,
            )
            .map_err(OpenErrInner::Json)?,
            root,
        })
    }
    pub fn create(root: &Path, project: Config) -> Result<Self, CreateError> {
        fn write_code(
            root: BorrowedFd,
            base: String,
            module: &str,
            code: &str,
        ) -> Result<CString, io::Error> {
            let mut rel_path = base;
            for m in module.split('.') {
                rel_path.push('/');
                rel_path.push_str(m);
            }
            rel_path.push_str(".hs");

            fs::mkdir_all_at(root, Path::new(rel_path.as_str()).parent().unwrap())?;
            let path = CString::new(rel_path)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            fs::write(root, &path, code)?;
            Ok(path)
        }
        let root = fs::open_dirfd(root).map_err(CreateErrorInner::Io)?;

        let code_mod = module_name(project.code)
            .ok_or_else(|| CreateErrorInner::UnknownCode(project.code.to_string()))?;
        let code_path = write_code(root.as_fd(), "src".into(), code_mod, project.code)
            .map_err(CreateErrorInner::WriteCode)?;

        let test_mod = module_name(project.fixture)
            .ok_or_else(|| CreateErrorInner::UnknownTest(project.fixture.to_string()))?;
        let test_path = write_code(
            root.as_fd(),
            "test/sample".into(),
            test_mod,
            project.fixture,
        )
        .map_err(CreateErrorInner::WriteCode)?;

        fs::write(
            root.as_fd(),
            c"test/sample/Main.hs",
            format!(
                include_str!("./haskell/Test_Main.hs",),
                test_module = test_mod
            ),
        )
        .map_err(CreateErrorInner::Io)?;

        let state = State {
            code_path,
            fixture_path: test_path,
        };
        fs::write(
            root.as_fd(),
            STATE_FILE,
            serde_json::to_vec(&state).unwrap(),
        )
        .map_err(CreateErrorInner::Io)?;

        fs::write(
            root.as_fd(),
            format!("{}.cabal", project.slug),
            format!(
                include_str!("./haskell/challenge.cabal"),
                package = project.slug,
                code_module = code_mod,
                test_module = test_mod
            ),
        )
        .map_err(CreateErrorInner::Io)?;

        fs::write(
            root.as_fd(),
            c"cabal.project.local",
            include_str!("./haskell/cabal.project.local.in"),
        )
        .map_err(CreateErrorInner::Io)?;

        Ok(Self { root, state })
    }
}
impl WorkspaceObject for Haskell {
    fn get_code(&self) -> Result<Code, io::Error> {
        Ok(Code {
            solution: fs::read_to_string(self.root.as_fd(), &self.state.code_path)?,
            fixture: fs::read_to_string(self.root.as_fd(), &self.state.fixture_path)?,
        })
    }
    fn clean_build(&self) -> Result<(), io::Error> {
        call_command_at(self.root.as_fd(), "cabal", ["clean"])
    }
    fn clean_session(&self) -> Result<(), io::Error> {
        fs::remove_at(self.root.as_fd(), STATE_FILE)?;
        fs::remove_at(self.root.as_fd(), c"cabal.project.local")?;
        fs::remove_at(self.root.as_fd(), c"cabal.project.local~")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    mod mod_name {
        use super::super::module_name;

        fn test_module(code: &str, exp: &str) {
            assert_eq!(module_name(code).unwrap(), exp);
        }

        #[test]
        fn simple() {
            test_module("module M1.M2.M3 where", "M1.M2.M3");
            test_module("module M1.M2.M3 (f1, f2, f3) where", "M1.M2.M3");
        }

        #[test]
        fn multiline_name() {
            test_module("module M1.M2\n    ( f1, f2, f3) where", "M1.M2");
        }

        #[test]
        fn extension() {
            test_module(
                "{-# LANGUAGE LambdaCase #-}\n\
                {-# LANGUAGE TupleSections #-}\n\
                module M1.M2 where",
                "M1.M2",
            )
        }

        #[test]
        fn comment() {
            test_module(
                "-- comment\n\
                module M1.M2 where",
                "M1.M2",
            )
        }
    }
}
