use std::{ffi::CStr, path::Path};

use rustix::{
    fd::{AsFd, OwnedFd},
    io::Errno,
};

use crate::{
    util::{call_command_at, fs},
    Code, Config, WorkspaceObject,
};

const CODE_PATH: &CStr = c"src/main/kotlin/library.kt";
const TEST_PATH: &CStr = c"src/test/kotlin/sample.kt";

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct OpenError(Errno);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct CreateError(#[from] Errno);

pub struct Kotlin {
    root: OwnedFd,
}
impl Kotlin {
    pub fn open(root: &Path) -> Result<Self, OpenError> {
        Ok(Self {
            root: fs::open_dirfd(root).map_err(OpenError)?,
        })
    }
    pub fn create(root: &Path, project: Config, patch: u16) -> Result<Self, CreateError> {
        let root = fs::open_dirfd(root)?;

        fs::write(
            root.as_fd(),
            c"build.gradle.kts",
            format!(
                include_str!("./kotlin/build.gradle.kts"),
                kotlin_version = format_args!("{}.{patch}", project.version_id)
            ),
        )?;

        fs::mkdirat(root.as_fd(), c"src")?;

        fs::mkdirat(root.as_fd(), c"src/main")?;
        fs::mkdirat(root.as_fd(), c"src/main/kotlin")?;
        fs::write(root.as_fd(), CODE_PATH, project.code)?;

        fs::mkdirat(root.as_fd(), c"src/test")?;
        fs::mkdirat(root.as_fd(), c"src/test/kotlin")?;
        fs::write(root.as_fd(), TEST_PATH, project.fixture)?;

        Ok(Self { root })
    }
}
impl WorkspaceObject for Kotlin {
    fn get_code(&self) -> Result<crate::Code, std::io::Error> {
        Ok(Code {
            solution: fs::read_to_string(self.root.as_fd(), CODE_PATH)?,
            fixture: fs::read_to_string(self.root.as_fd(), TEST_PATH)?,
        })
    }
    fn clean_build(&self) -> Result<(), std::io::Error> {
        call_command_at(self.root.as_fd(), "gradle", ["clean"])?;
        Ok(())
    }
    fn clean_session(&self) -> Result<(), std::io::Error> {
        fs::remove_dir_all_at(self.root.as_fd(), c".gradle")?;
        fs::remove_dir_all_at(self.root.as_fd(), c".kotlin")?;
        Ok(())
    }
}
