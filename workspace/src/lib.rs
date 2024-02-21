use std::{error, io, path::Path};

pub struct Code {
    pub solution: String,
    pub fixture: String,
}

pub trait WorkspaceObject<E = io::Error>
where
    E: error::Error,
{
    fn root(&self) -> &Path;
    fn get_code(&self) -> Result<Code, E>;
    fn clean(&self) -> Result<(), E>;
}

pub mod rust;
pub use rust::Rust;

pub mod haskell;
pub use haskell::Haskell;

pub mod coq;
pub use coq::Coq;
