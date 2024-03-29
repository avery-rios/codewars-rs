extern crate codewars_api as api;
extern crate codewars_solution as solution;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::Path;
use tokio::runtime;

use codewars_solution::index;
use codewars_types::{KataId, KnownLangId};

mod command;
use command::{next_cmd, print_err, CmdEnv, CmdState};

mod kata;

mod session;

mod suggest;

mod rank;

mod user;

mod file_list;

#[derive(Subcommand)]
enum KataCmd {
    /// Get kata information
    Get {
        id: KataId,
    },
    Train {
        id: KataId,
        lang: KnownLangId,
    },
    /// Suggest kata
    Suggest {
        lang: KnownLangId,
        strategy: Option<suggest::Strategy>,
    },
}
impl KataCmd {
    fn run(self, env: &CmdEnv, state: &mut CmdState) -> Result<()> {
        match self {
            Self::Get { id } => {
                env.runtime.block_on(kata::get_kata(
                    &id,
                    &env.api_client,
                    state.index_mut(),
                    Path::new(&env.root),
                ))?;
                state.index_dirty = true;
                Ok(())
            }
            Self::Train { id, lang } => session::start_session(env, state, id, lang),
            Self::Suggest { lang, strategy } => {
                suggest::start_suggest(env, state, lang, strategy.unwrap_or_default())
            }
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
                *state.index_mut() = index::Index::build(&env.root)?;
            }
            Self::Save => {
                state.index().write(&env.index_path)?;
                state.index_dirty = false;
            }
        }
        Ok(())
    }
}

#[derive(Subcommand)]
enum SessionCmd {
    Open { path: String },
}
impl SessionCmd {
    fn run(self, env: &CmdEnv, state: &mut CmdState) -> Result<()> {
        match self {
            Self::Open { path } => session::open_session(env, state, path),
        }
    }
}

#[derive(Subcommand)]
enum UserCmd {
    Info { id: String },
}
impl UserCmd {
    fn run(self, env: &CmdEnv, _: &mut CmdState) -> Result<()> {
        match self {
            Self::Info { id } => user::show_user(
                &env.runtime
                    .block_on(env.api_client.get_user(&id))
                    .context("failed to get user")?,
            ),
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
    #[command(subcommand)]
    Session(SessionCmd),
    #[command(subcommand)]
    User(UserCmd),
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
            Self::Session(s) => s.run(env, state)?,
            Self::User(u) => u.run(env, state)?,
            Self::Exit { no_save } => {
                if !no_save && state.index_dirty {
                    state
                        .index()
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
    #[arg(long)]
    login: bool,
    #[arg(long)]
    log_request: bool,
    #[arg(long, env = "CW_SESSION_ID")]
    session_id: Option<String>,
    #[arg(long, env = "CW_USER_TOKEN")]
    user_token: Option<String>,
    /// file list options
    #[arg(long, env = "CW_LIST_OPTIONS")]
    list_option: Option<String>,
    #[arg(long, default_value = ".")]
    root: String,
    #[arg(long)]
    workspace: String,
    #[command(subcommand)]
    command: Option<Command>,
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let env = {
        let runtime = runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("failed to create runtime")?;
        CmdEnv {
            index_path: Path::new(&cli.root).join(index::INDEX_FILE),
            root: cli.root,
            workspace: cli.workspace,
            api_client: codewars_api::Client::new(),
            unofficial_client: if cli.login {
                println!("Login into codewars");
                Some(
                    runtime
                        .block_on(codewars_unofficial::Client::init(
                            cli.log_request,
                            &cli.session_id.context("Missing session id")?,
                            &cli.user_token.context("Missing user token")?,
                        ))
                        .context("failed to login to codewars")?,
                )
            } else {
                None
            },
            list_option: match cli.list_option {
                Some(o) => {
                    file_list::Options::parse(&o).context("failed to parse file list options")?
                }
                None => file_list::Options::default(),
            },
            runtime,
        }
    };
    let mut state = CmdState::new(if env.index_path.exists() {
        index::Index::open(&env.index_path).context("failed to open index")?
    } else {
        index::Index::new()
    });
    match cli.command {
        Some(c) => {
            if let Err(e) = c.run(&env, &mut state) {
                print_err(e)
            }
            if state.index_dirty {
                if let Err(e) = state.index().write(&env.index_path) {
                    print_err(anyhow::Error::new(e).context("failed to save index"));
                }
            }
        }
        None => {
            let mut editor = command::new_editor().context("failed to create line editor")?;
            loop {
                match next_cmd::<Command>("codewars> ", &mut editor).run(&env, &mut state) {
                    Ok(true) => (),
                    Ok(false) => break,
                    Err(e) => print_err(e),
                }
            }
        }
    }
    Ok(())
}
