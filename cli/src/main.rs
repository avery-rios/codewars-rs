extern crate codewars_api as api;
extern crate codewars_solution as solution;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::{fs, path::Path};
use tokio::runtime;

use codewars_types::KataId;

mod command;
use command::{next_cmd, print_err, CmdEnv};

#[derive(Subcommand)]
enum KataCmd {
    /// Get kata information
    Get { id: KataId },
}
async fn get_kata(id: &KataId, client: &api::Client, root: &Path) -> Result<()> {
    use solution::*;
    fn to_author(auth: api::Author) -> Author {
        Author {
            username: auth.username,
            url: auth.url,
        }
    }
    fn to_approve(
        rank: Option<api::KataRank>,
        approver: Option<api::Author>,
        approved_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Option<KataApprove> {
        Some(KataApprove {
            rank: rank?.id,
            approver: match approver {
                Some(a) => to_author(a),
                None => return None,
            },
            approved_at: approved_at?,
        })
    }

    let kata = client
        .get_challenge(id)
        .await
        .context("failed to get kata info")?;
    let info = KataInfo {
        name: kata.name,
        id: kata.id,
        slug: kata.slug,
        url: kata.url,
        approve: to_approve(kata.rank, kata.approved_by, kata.approved_at),
        created_by: to_author(kata.created_by),
        created_at: kata.published_at,
        category: kata.category,
        tags: kata.tags,
    };
    let kata_root = root.join(kata_dir(&info.id, &info.slug));
    if kata_root.exists() {
        anyhow::bail!("Kata {} is already fetched", id);
    }
    fs::create_dir(&kata_root).context("failed to create kata dir")?;
    println!("Kata {} will be saved to {}", id, kata_root.display());
    write_kata(
        kata_root,
        &Metadata {
            version: Version::CURRENT,
            api_version: ApiVersion::CURRENT,
            created_at: chrono::Local::now().fixed_offset(),
            updated_at: Vec::new(),
        },
        &info,
        &kata.description,
    )
    .context("failed to write kata")
}
impl KataCmd {
    fn run(self, env: &CmdEnv) -> Result<()> {
        match self {
            Self::Get { id } => {
                env.runtime
                    .block_on(get_kata(&id, &env.api_client, Path::new(&env.root)))
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
