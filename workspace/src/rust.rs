use std::{borrow::Cow, ffi::CStr, io, path::Path};

use rustix::{
    fd::{AsFd, OwnedFd},
    io::Errno,
};

use crate::{
    util::{call_command_at, fs},
    Code, Config, WorkspaceObject,
};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] Errno);

pub struct Rust {
    root: OwnedFd,
}
const CODE_PATH: &CStr = c"src/lib.rs";
const TEST_PATH: &CStr = c"tests/sample.rs";

impl Rust {
    pub fn open(root: &Path) -> Result<Self, Error> {
        Ok(Self {
            root: fs::open_dirfd(root)?,
        })
    }

    pub fn create(root: impl AsRef<Path>, project: Config) -> Result<Self, Error> {
        let root = fs::open_dirfd(root.as_ref())?;

        fs::mkdirat(root.as_fd(), c"src")?;
        fs::write(root.as_fd(), CODE_PATH, project.code)?;

        let crate_name = if project.slug.starts_with(|c: char| !c.is_alphabetic()) {
            Cow::Owned(format!("cw-{}", project.slug))
        } else {
            Cow::Borrowed(project.slug)
        };
        let crate_name_rs = crate_name.replace('-', "_");

        fs::mkdirat(root.as_fd(), c"tests")?;
        fs::write(
            root.as_fd(),
            TEST_PATH,
            format!(
                "#[cfg(feature = \"local\")]\n\
                use {crate_name_rs}::*;\n\
                \n\
                {}",
                project.fixture
            ),
        )?;

        fs::mkdirat(root.as_fd(), c".cargo")?;
        fs::write(
            root.as_fd(),
            c".cargo/config.toml",
            include_str!("./rust/profile.toml"),
        )?;

        fs::write(
            root.as_fd(),
            c"Cargo.toml",
            format!(
                include_str!("./rust/Cargo.toml"),
                crate_name = crate_name,
                rust_version = project.version_id
            ),
        )?;

        Ok(Self { root })
    }
}

impl WorkspaceObject for Rust {
    fn get_code(&self) -> Result<Code, io::Error> {
        Ok(Code {
            solution: fs::read_to_string(self.root.as_fd(), CODE_PATH)?,
            fixture: fs::read_to_string(self.root.as_fd(), TEST_PATH)?,
        })
    }
    fn clean_build(&self) -> Result<(), io::Error> {
        call_command_at(self.root.as_fd(), "cargo", ["clean"])
    }
    fn clean_session(&self) -> Result<(), io::Error> {
        fs::remove_dir_all_at(self.root.as_fd(), c".cargo")?;
        fs::remove_at(self.root.as_fd(), c"Cargo.lock")?;
        Ok(())
    }
}
