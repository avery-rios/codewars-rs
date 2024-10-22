use std::{ffi::CStr, io, path::Path};

use rustix::{
    fd::{AsFd, OwnedFd},
    fs::{Mode, OFlags},
};

use crate::{
    util::{call_command_at, fs},
    Code, Config, WorkspaceObject,
};

pub struct Coq {
    root: OwnedFd,
}

macro_rules! file_name {
    (preloaded) => {
        "Preloaded.v"
    };
    (code) => {
        "Solution.v"
    };
    (sample) => {
        "Test.v"
    };
}

const PRELOADED_FILE: &CStr = c"Preloaded.v";
const CODE_FILE: &CStr = c"Solution.v";
const FIXTURE_FILE: &CStr = c"Test.v";

impl Coq {
    pub fn create(root: &Path, project: Config) -> io::Result<Self> {
        let root = fs::open_dirfd(root)?;

        fs::write(root.as_fd(), CODE_FILE, project.code)?;
        fs::write(root.as_fd(), FIXTURE_FILE, project.fixture)?;
        fs::write(
            root.as_fd(),
            c"_CoqProject",
            include_str!("./coq/_CoqProject"),
        )?;
        fs::write(
            root.as_fd(),
            c"Makefile",
            format!(
                include_str!("./coq/Makefile"),
                files = if project.has_preload {
                    concat!(file_name!(preloaded), " ", file_name!(code))
                } else {
                    file_name!(code)
                }
            ),
        )?;

        if project.has_preload {
            // create empty file
            rustix::fs::openat(
                root.as_fd(),
                PRELOADED_FILE,
                OFlags::CREATE | OFlags::CLOEXEC,
                Mode::from_raw_mode(0o666),
            )?;
        }

        Ok(Self { root })
    }
    pub fn open(root: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self {
            root: fs::open_dirfd(root.as_ref())?,
        })
    }
}
impl WorkspaceObject for Coq {
    fn get_code(&self) -> Result<crate::Code, io::Error> {
        Ok(Code {
            solution: fs::read_to_string(self.root.as_fd(), CODE_FILE)?,
            fixture: fs::read_to_string(self.root.as_fd(), FIXTURE_FILE)?,
        })
    }
    fn clean_build(&self) -> Result<(), io::Error> {
        call_command_at(self.root.as_fd(), "make", ["clean"])
    }
    fn clean_session(&self) -> Result<(), io::Error> {
        Ok(())
    }
}
