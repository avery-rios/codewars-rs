extern crate codewars_api as api;
extern crate codewars_solution as solution;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::Path;
use tokio::runtime;

use codewars_solution::index;
use codewars_types::KataId;

mod command;
use command::{next_cmd, print_err, CmdEnv, CmdState};

mod kata;

#[derive(Subcommand)]
enum KataCmd {
    /// Get kata information
    Get { id: KataId },
}
impl KataCmd {
    fn run(self, env: &CmdEnv, state: &mut CmdState) -> Result<()> {
        match self {
            Self::Get { id } => env.runtime.block_on(kata::get_kata(
                &id,
                &env.api_client,
                &mut state.index,
                Path::new(&env.root),
            )),
        }
    }
}

#[derive(Subcommand)]
enum IndexCmd {
    /// Rebuild kata index
    Rebuild,
    /// save kata index
    Save,
}
impl IndexCmd {
    fn run(self, env: &CmdEnv, state: &mut CmdState) -> Result<()> {
        match self {
            Self::Rebuild => {
                state.index = index::Index::build(&env.root)?;
            }
            Self::Save => {
                state.index.write(&env.index_path)?;
            }
        }
        Ok(())
    }
}

#[derive(Subcommand)]
enum Command {
    #[command(subcommand)]
    Kata(KataCmd),
    #[command(subcommand)]
    Index(IndexCmd),
    /// exit codewars cli
    Exit {
        #[arg(long)]
        no_save: bool,
    },
}
impl Command {
    fn run(self, env: &CmdEnv, state: &mut CmdState) -> Result<bool> {
        match self {
            Self::Kata(k) => k.run(env, state)?,
            Self::Index(idx) => idx.run(env, state)?,
            Self::Exit { no_save } => {
                if !no_save {
                    state
                        .index
                        .write(&env.index_path)
                        .context("failed to write index")?;
                }
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = ".")]
    root: String,
    #[command(subcommand)]
    command: Option<Command>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let env = {
        let runtime = runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("failed to create runtime")?;
        CmdEnv {
            index_path: Path::new(&cli.root).join(index::INDEX_FILE),
            root: cli.root,
            api_client: codewars_api::Client::new(),
            runtime,
        }
    };
    let mut state = CmdState {
        index: if env.index_path.exists() {
            index::Index::open(&env.index_path).context("failed to open index")?
        } else {
            index::Index::new()
        },
    };
    let mut editor = command::new_editor().context("failed to create line editor")?;
    match cli.command {
        Some(c) => {
            if let Err(e) = c.run(&env, &mut state) {
                print_err(e)
            }
            if let Err(e) = state.index.write(&env.index_path) {
                print_err(anyhow::Error::new(e).context("failed to save index"));
            }
        }
        None => loop {
            match next_cmd::<Command>("codewars> ", &mut editor).run(&env, &mut state) {
                Ok(true) => (),
                Ok(false) => break,
                Err(e) => print_err(e),
            }
        },
    }
    Ok(())
}
