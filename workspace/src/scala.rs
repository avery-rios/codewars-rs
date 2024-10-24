use std::{ffi::CStr, path::Path};

use rustix::{
    fd::{AsFd, BorrowedFd, OwnedFd},
    io::Errno,
};

use crate::{
    util::{call_command_at, fs},
    Config, WorkspaceObject,
};

const CODE_PATH: &CStr = c"src/solution.scala";
const TEST_PATH: &CStr = c"sample/src/test.scala";

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct OpenError(#[from] Errno);

#[derive(Debug, thiserror::Error)]
enum ScalafmtError {
    #[error("failed to run scalafmt")]
    Run(#[source] std::io::Error),
    #[error("scalafmt returned: {0:?}")]
    Status(std::process::ExitStatus),
    #[error("scalafmt stdout is not utf8 string")]
    BinaryOutput(#[source] std::string::FromUtf8Error),
    #[error("unknown scalafmt output: {0:?}")]
    InvalidData(String),
}

#[derive(Debug, thiserror::Error)]
enum CreateErrorInner {
    #[error("unsupported scala version")]
    UnsupportedVersion,
    #[error("io error")]
    Io(#[source] Errno),
    #[error("failed to get scalafmt version")]
    ScalaFmtVersion(#[source] ScalafmtError),
}
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct CreateError(#[from] CreateErrorInner);

enum Version {
    Scala2_13,
    Scala3_0,
}

fn scalafmt_version() -> Result<String, ScalafmtError> {
    let output = std::process::Command::new("scalafmt")
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(ScalafmtError::Run)?;
    if !output.status.success() {
        return Err(ScalafmtError::Status(output.status));
    }
    let stdout = String::from_utf8(output.stdout).map_err(ScalafmtError::BinaryOutput)?;
    match stdout.trim().strip_prefix("scalafmt").map(str::trim) {
        Some(v) => Ok(v.to_string()),
        None => Err(ScalafmtError::InvalidData(stdout)),
    }
}

pub struct Scala {
    root: OwnedFd,
}
impl Scala {
    pub fn open(root: &Path) -> Result<Self, OpenError> {
        Ok(Self {
            root: fs::open_dirfd(root)?,
        })
    }
    pub fn create(
        root: &Path,
        project: Config,
        local_scala_version: &str,
    ) -> Result<Self, CreateError> {
        let version = match project.version_id {
            "2.13" => Version::Scala2_13,
            "3.0" => Version::Scala3_0,
            _ => return Err(CreateError(CreateErrorInner::UnsupportedVersion)),
        };
        let scalafmt = scalafmt_version().map_err(CreateErrorInner::ScalaFmtVersion)?;

        let root = fs::open_dirfd(root).map_err(CreateErrorInner::Io)?;

        fn inner(
            root: BorrowedFd,
            version: Version,
            project: Config,
            local_scala_version: &str,
            scalafmt: &str,
        ) -> Result<(), Errno> {
            fs::mkdirat(root, c"src")?;
            fs::write(root, CODE_PATH, project.code)?;

            fs::mkdirat(root, c"sample")?;
            fs::mkdirat(root, c"sample/src")?;
            fs::write(root, TEST_PATH, project.fixture)?;

            fs::write(
                root,
                c".scalafmt.conf",
                format!(
                    include_str!("./scala/scalafmt.conf"),
                    scalafmt_version = scalafmt,
                    scala_version = match version {
                        Version::Scala2_13 => "scala213",
                        Version::Scala3_0 => "scala3",
                    }
                ),
            )?;

            fs::write(
                root,
                c"build.sc",
                format!(
                    include_str!("./scala/build.sc"),
                    scala_version = local_scala_version,
                    test_framework = match version {
                        Version::Scala2_13 => "org.scalatest::scalatest:3.0.8",
                        Version::Scala3_0 => "org.scalatest::scalatest:3.2.10",
                    }
                ),
            )?;

            Ok(())
        }
        inner(
            root.as_fd(),
            version,
            project,
            local_scala_version,
            &scalafmt,
        )
        .map_err(CreateErrorInner::Io)?;

        Ok(Scala { root })
    }
}
impl WorkspaceObject for Scala {
    fn get_code(&self) -> Result<crate::Code, std::io::Error> {
        Ok(crate::Code {
            solution: fs::read_to_string(self.root.as_fd(), CODE_PATH)?,
            fixture: fs::read_to_string(self.root.as_fd(), TEST_PATH)?,
        })
    }
    fn clean_build(&self) -> Result<(), std::io::Error> {
        call_command_at(self.root.as_fd(), "mill", ["clean"])?;
        Ok(())
    }
    fn clean_session(&self) -> Result<(), std::io::Error> {
        fs::remove_dir_all_at(self.root.as_fd(), c".bloop")?;
        fs::remove_dir_all_at(self.root.as_fd(), c".metals")?;
        fs::remove_dir_all_at(self.root.as_fd(), c".vscode")?;
        fs::remove_dir_all_at(self.root.as_fd(), c"out")?;
        Ok(())
    }
}
