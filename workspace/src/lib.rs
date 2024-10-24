use std::{error, io};

mod util;

pub struct Config<'a> {
    pub version_id: &'a str,
    pub slug: &'a str,
    pub code: &'a str,
    pub fixture: &'a str,
    pub has_preload: bool,
}
pub struct Code {
    pub solution: String,
    pub fixture: String,
}

pub trait WorkspaceObject<E = io::Error>
where
    E: error::Error,
{
    fn get_code(&self) -> Result<Code, E>;
    fn clean_build(&self) -> Result<(), E>;
    fn clean_session(&self) -> Result<(), E>;
}

pub mod rust;
pub use rust::Rust;

pub mod haskell;
pub use haskell::Haskell;

pub mod java;
pub use java::Java;

pub mod kotlin;
pub use kotlin::Kotlin;

pub mod coq;
pub use coq::Coq;

pub mod typescript;
pub use typescript::TypeScript;

pub mod scala;
pub use scala::Scala;
