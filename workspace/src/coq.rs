use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::{util::call_command_in, Code, WorkspaceObject};

pub struct Coq {
    root: PathBuf,
    code_path: PathBuf,
    fixture: String,
}

const CODE_FILE: &str = "Challenge.v";
const FIXTURE_FILE: &str = "Test.v";

impl Coq {
    pub fn create(mut root: PathBuf, code: &str, test: &str) -> io::Result<Self> {
        let code_path = root.join(CODE_FILE);
        fs::write(&code_path, code)?;

        root.push(FIXTURE_FILE);
        fs::write(&root, test)?;
        root.pop();

        root.push("_CoqProject");
        fs::write(&root, include_str!("./coq/_CoqProject"))?;
        root.pop();

        root.push("Makefile");
        fs::write(
            &root,
            format!(include_str!("./coq/Makefile"), files = CODE_FILE),
        )?;
        root.pop();

        Ok(Self {
            root,
            code_path,
            fixture: test.to_string(),
        })
    }
    pub fn open(root: impl AsRef<Path>) -> io::Result<Self> {
        let root = root.as_ref();
        Ok(Self {
            root: root.to_path_buf(),
            code_path: root.join(CODE_FILE),
            fixture: fs::read_to_string(root.join(FIXTURE_FILE))?,
        })
    }
}
impl WorkspaceObject for Coq {
    fn root(&self) -> &Path {
        self.root.as_path()
    }
    fn get_code(&self) -> Result<crate::Code, io::Error> {
        Ok(Code {
            solution: fs::read_to_string(&self.code_path)?,
            fixture: self.fixture.clone(),
        })
    }
    fn clean_build(&self) -> Result<(), io::Error> {
        call_command_in(&self.root, "make", ["clean"])
    }
    fn clean_session(&self) -> Result<(), io::Error> {
        Ok(())
    }
}
