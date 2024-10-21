use std::{ffi::CStr, io, path::Path};

use rustix::{
    fd::{AsFd, OwnedFd},
    io::Errno,
};

use crate::{
    util::{call_command_at, fs},
    Code, WorkspaceObject,
};

macro_rules! crate_name {
    () => {
        "challenge"
    };
}

const CARGO_1_62: &str = concat!(
    "[package]\n",
    concat!("name = \"", crate_name!(), "\"\n"),
    "version = \"0.1.0\"\n",
    "edition = \"2021\"\n",
    "rust-version = \"1.62\"\n",
    "\n",
    "[features]\n",
    "local = []\n",
    "default = [\"local\"]"
);

const CRATE_NAME: &str = crate_name!();

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] Errno);

pub struct Rust {
    root: OwnedFd,
}
const CODE_PATH: &CStr = c"src/lib.rs";
const TEST_PATH: &CStr = c"tests/sample.rs";

impl Rust {
    pub const VERSION: &'static str = "1.62";

    pub fn open(root: &Path) -> Result<Self, Error> {
        Ok(Self {
            root: fs::open_dirfd(root)?,
        })
    }

    pub fn create(root: impl AsRef<Path>, code: &str, test: &str) -> Result<Self, Error> {
        let root = fs::open_dirfd(root.as_ref())?;

        fs::mkdirat(root.as_fd(), c"src")?;
        fs::write(root.as_fd(), CODE_PATH, code)?;

        fs::mkdirat(root.as_fd(), c"tests")?;
        fs::write(
            root.as_fd(),
            TEST_PATH,
            format!(
                "#[cfg(feature = \"local\")]\n\
                use {CRATE_NAME}::*;\n\
                \n\
                {test}"
            ),
        )?;

        fs::mkdirat(root.as_fd(), c".cargo")?;
        fs::write(
            root.as_fd(),
            c".cargo/config.toml",
            include_str!("./rust/profile.toml"),
        )?;

        fs::write(root.as_fd(), c"Cargo.toml", CARGO_1_62)?;

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
