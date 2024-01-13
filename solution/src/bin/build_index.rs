use std::{env, io, path::PathBuf};

use codewars_solution::index;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("too few args, expect path")]
    TooFewArg,
    #[error("failed to build index")]
    Build(
        #[from]
        #[source]
        index::BuildError,
    ),
    #[error("failed to write index file")]
    Write(
        #[from]
        #[source]
        io::Error,
    ),
}

// TODO: use std error reporter when stabilized
fn main() -> Result<(), Error> {
    let mut path = {
        let mut args = env::args_os().fuse();
        let _ = args.next();
        PathBuf::from(args.next().ok_or(Error::TooFewArg)?)
    };

    let index = index::Index::build(&path)?;

    path.push(index::INDEX_FILE);
    index.write(path).map_err(Error::Write)
}
