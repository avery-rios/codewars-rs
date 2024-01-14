extern crate codewars_api as api;
extern crate codewars_solution as solution;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::Path;
use tokio::runtime;

use codewars_types::KataId;

mod command;
use command::{next_cmd, print_err, CmdEnv};

mod kata;

#[derive(Subcommand)]
enum KataCmd {
    /// Get kata information
    Get { id: KataId },
}
impl KataCmd {
    fn run(self, env: &CmdEnv) -> Result<()> {
        match self {
            Self::Get { id } => {
                env.runtime
                    .block_on(kata::get_kata(&id, &env.api_client, Path::new(&env.root)))
            }
        }
    }
}

#[derive(Subcommand)]
enum Command {
    #[command(subcommand)]
    Kata(KataCmd),
    /// exit codewars cli
    Exit,
}
impl Command {
    fn run(self, env: &CmdEnv) -> Result<bool> {
        match self {
            Self::Kata(k) => k.run(env)?,
            Self::Exit => return Ok(false),
        }
        Ok(true)
    }
}

#[derive(Parser)]
struct Cli {
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
            root: String::from("."),
            api_client: codewars_api::Client::new(),
            runtime,
        }
    };
    let mut editor = command::new_editor().context("failed to create line editor")?;
    match cli.command {
        Some(c) => {
            if let Err(e) = c.run(&env) {
                print_err(e)
            }
        }
        None => loop {
            match next_cmd::<Command>("codewars> ", &mut editor).run(&env) {
                Ok(true) => (),
                Ok(false) => break,
                Err(e) => print_err(e),
            }
        },
    }
    Ok(())
}
