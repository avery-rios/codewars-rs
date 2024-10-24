extern crate alloc;

use alloc::ffi::{CString, NulError};
use std::{ffi::CStr, path::Path};

use rustix::{
    fd::{AsFd, BorrowedFd, OwnedFd},
    io::Errno,
};
use serde::{Deserialize, Serialize};

use crate::{
    util::{call_command_at, fs},
    Code, Config, WorkspaceObject,
};

const STATE_PATH: &CStr = c"state.json";

fn public_class(mut code: &str) -> Option<&str> {
    while !code.is_empty() {
        let (_, t) = code.split_once("public")?;
        match t.trim_start().strip_prefix("class") {
            Some(t) => {
                return Some(
                    t.trim_start()
                        .split_once(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$'))?
                        .0,
                )
            }
            None => code = t,
        }
    }
    None
}

#[derive(Serialize, Deserialize)]
struct State<'a> {
    main_class: &'a str,
    test_class: &'a str,
}
impl<'a> State<'a> {
    fn path(&self) -> Result<(CString, CString), std::ffi::NulError> {
        Ok((
            CString::new(format!("src/main/java/{}.java", self.main_class))?,
            CString::new(format!("src/test/java/{}.java", self.test_class))?,
        ))
    }
}

#[derive(Debug, thiserror::Error)]
enum OpenErrorInner {
    #[error("io error")]
    Io(#[source] Errno),
    #[error("failed to parse json")]
    Json(#[source] serde_json::Error),
    #[error("class name contains nul")]
    InvalidClassName(#[source] NulError),
}
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct OpenError(#[from] OpenErrorInner);

#[derive(Debug, thiserror::Error)]
enum CreateErrorInner {
    #[error("io error")]
    Io(#[source] Errno),
    #[error("class name contains nul")]
    InvalidClassName(#[source] NulError),
    #[error("failed to get public class name")]
    UnknownClassName,
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct CreateError(#[from] CreateErrorInner);

pub struct Java {
    root: OwnedFd,
    code_path: CString,
    test_path: CString,
}
impl Java {
    pub fn open(root: &Path) -> Result<Self, OpenError> {
        let root = fs::open_dirfd(root).map_err(OpenErrorInner::Io)?;

        let state_buf = fs::read(root.as_fd(), STATE_PATH).map_err(OpenErrorInner::Io)?;
        let state: State = serde_json::from_slice(&state_buf).map_err(OpenErrorInner::Json)?;

        let (code_path, test_path) = state.path().map_err(OpenErrorInner::InvalidClassName)?;

        Ok(Self {
            root,
            code_path,
            test_path,
        })
    }
    pub fn create(root: &Path, project: Config) -> Result<Self, CreateError> {
        let root = fs::open_dirfd(root).map_err(CreateErrorInner::Io)?;

        let state = State {
            main_class: public_class(&project.code).ok_or(CreateErrorInner::UnknownClassName)?,
            test_class: public_class(&project.fixture).ok_or(CreateErrorInner::UnknownClassName)?,
        };
        let (code_path, test_path) = state.path().map_err(CreateErrorInner::InvalidClassName)?;

        fn inner(
            root: BorrowedFd,
            project: &Config,
            state: &State,
            code_path: &CStr,
            test_path: &CStr,
        ) -> Result<(), Errno> {
            fs::mkdirat(root, c"src")?;

            fs::mkdirat(root, c"src/main")?;
            fs::mkdirat(root, c"src/main/java")?;
            fs::write(root, code_path, project.code)?;

            fs::mkdirat(root, c"src/test")?;
            fs::mkdirat(root, c"src/test/java")?;
            fs::write(root, test_path, project.fixture)?;

            fs::write(
                root,
                c"build.gradle.kts",
                include_str!("./java/build.gradle.kts"),
            )?;

            fs::write(root, STATE_PATH, serde_json::to_vec(state).unwrap())?;

            Ok(())
        }

        inner(root.as_fd(), &project, &state, &code_path, &test_path)
            .map_err(CreateErrorInner::Io)?;

        Ok(Self {
            root,
            code_path,
            test_path,
        })
    }
}
impl WorkspaceObject for Java {
    fn get_code(&self) -> Result<crate::Code, std::io::Error> {
        Ok(Code {
            solution: fs::read_to_string(self.root.as_fd(), &self.code_path)?,
            fixture: fs::read_to_string(self.root.as_fd(), &self.test_path)?,
        })
    }
    fn clean_build(&self) -> Result<(), std::io::Error> {
        call_command_at(self.root.as_fd(), "gradle", ["clean"])?;
        Ok(())
    }
    fn clean_session(&self) -> Result<(), std::io::Error> {
        fs::remove_dir_all_at(self.root.as_fd(), c".gradle")?;
        fs::remove_at(self.root.as_fd(), STATE_PATH)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::public_class;

    #[test]
    fn simple_class() {
        assert_eq!(
            public_class("import pack;\npublic class Cls{}"),
            Some("Cls")
        )
    }
}
