use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::{Code, WorkspaceObject};

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

const CARGO_CONFIG: &str = include_str!("./rust/profile.toml");

const CRATE_NAME: &str = crate_name!();

fn proc_test(input: &str) -> String {
    format!(
        "#[cfg(feature = \"local\")]\nuse {}::*;\n\n{}",
        CRATE_NAME, input
    )
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct CreateError(#[from] io::Error);

pub struct Rust {
    root: PathBuf,
    code_path: PathBuf,
    example_test_path: PathBuf,
}
impl Rust {
    pub const VERSION: &str = "1.62";

    pub fn open(root: PathBuf) -> Self {
        Self {
            code_path: root.join("src/lib.rs"),
            example_test_path: root.join("tests/sample.rs"),
            root,
        }
    }

    pub fn create(root: impl AsRef<Path>, code: &str, test: &str) -> Result<Self, CreateError> {
        let root = root.as_ref();
        let code_path = {
            let mut dir = root.join("src");
            fs::create_dir_all(&dir)?;
            dir.push("lib.rs");
            dir
        };
        fs::write(&code_path, code)?;

        let example_test_path = {
            let mut dir = root.join("tests");
            fs::create_dir(&dir)?;
            dir.push("sample.rs");
            dir
        };
        fs::write(&example_test_path, proc_test(test))?;

        let mut tmp = root.to_path_buf();

        tmp.push(".cargo");
        fs::create_dir(&tmp)?;
        tmp.push("config.toml");
        fs::write(&tmp, CARGO_CONFIG)?;
        tmp.pop();
        tmp.pop();

        tmp.push("Cargo.toml");
        fs::write(&tmp, CARGO_1_62)?;
        tmp.pop();

        Ok(Self {
            root: root.to_path_buf(),
            code_path,
            example_test_path,
        })
    }
}

impl WorkspaceObject for Rust {
    fn root(&self) -> &Path {
        &self.root
    }
    fn get_code(&self) -> Result<Code, io::Error> {
        Ok(Code {
            solution: fs::read_to_string(&self.code_path)?,
            fixture: fs::read_to_string(&self.example_test_path)?,
        })
    }
    fn clean(&self) -> Result<(), io::Error> {
        let mut tmp = self.root.join(".cargo");
        if tmp.exists() {
            fs::remove_dir_all(&tmp)?;
        }
        tmp.pop();

        tmp.push("Cargo.lock");
        fs::remove_file(&tmp)?;
        Ok(())
    }
}
