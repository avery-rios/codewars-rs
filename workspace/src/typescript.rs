use std::{ffi::CStr, path::Path};

use rustix::{
    fd::{AsFd, OwnedFd},
    io::Errno,
};

use crate::{util::fs, Code, Config, WorkspaceObject};

const CODE_PATH: &CStr = c"src/index.ts";
const TEST_PATH: &CStr = c"test/sample.ts";

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct OpenError(Errno);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct CreateError(#[from] Errno);

pub struct TypeScript {
    root: OwnedFd,
}
impl TypeScript {
    pub fn open(path: &Path) -> Result<Self, OpenError> {
        Ok(Self {
            root: fs::open_dirfd(path).map_err(OpenError)?,
        })
    }
    pub fn create(root: &Path, config: Config) -> Result<Self, CreateError> {
        let root = fs::open_dirfd(root)?;

        fs::write(
            root.as_fd(),
            c"tsconfig.json",
            include_str!("./typescript/tsconfig.json"),
        )?;
        fs::write(
            root.as_fd(),
            c"package.json",
            format!(
                include_str!("./typescript/package.json.in"),
                package = config.slug,
                tsc_version = config.version_id
            ),
        )?;

        fs::mkdirat(root.as_fd(), c"src")?;
        fs::write(root.as_fd(), CODE_PATH, config.code)?;

        fs::mkdirat(root.as_fd(), c"test")?;
        rustix::fs::symlinkat(c"../src", root.as_fd(), c"test/solution")?;
        fs::write(root.as_fd(), TEST_PATH, config.fixture)?;

        Ok(Self { root })
    }
}
impl WorkspaceObject for TypeScript {
    fn get_code(&self) -> Result<crate::Code, std::io::Error> {
        Ok(Code {
            solution: fs::read_to_string(self.root.as_fd(), CODE_PATH)?,
            fixture: fs::read_to_string(self.root.as_fd(), TEST_PATH)?,
        })
    }
    fn clean_build(&self) -> Result<(), std::io::Error> {
        fs::remove_dir_all_at(self.root.as_fd(), c"dist")?;
        Ok(())
    }
    fn clean_session(&self) -> Result<(), std::io::Error> {
        fs::remove_dir_all_at(self.root.as_fd(), c"node_modules")?;
        fs::remove_at(self.root.as_fd(), c"pnpm-lock.yaml")?;
        Ok(())
    }
}
