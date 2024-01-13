extern crate codewars_api as api;
extern crate codewars_solution as solution;

use anyhow::{Context, Result};
use api::Client;
use clap::{FromArgMatches, Parser, Subcommand};
use rustyline::Editor;
use std::{fs, path::Path};
use tokio::runtime::{self, Runtime};

use codewars_types::KataId;

#[derive(Subcommand)]
enum KataCmd {
    /// Get kata information
    Get { id: KataId },
}
async fn get_kata(id: &KataId, client: &Client, root: &Path) -> Result<()> {
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
    let kata_root = kata_path(root, &info);
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
    async fn run(self, client: &Client, root: &Path) -> Result<()> {
        match self {
            Self::Get { id } => get_kata(&id, client, root).await,
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
    async fn run(self, client: &Client, root: &Path) -> Result<bool> {
        match self {
            Self::Kata(k) => k.run(client, root).await?,
            Self::Exit => return Ok(false),
        }
        Ok(true)
    }
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

type LineEditor = Editor<(), rustyline::history::MemHistory>;

fn read_exec(client: &Client, runtime: &Runtime, editor: &mut LineEditor) -> Result<bool> {
    let inputs = shlex::split(&editor.readline("codewars> ")?).context("failed to split input")?;
    let c = Command::augment_subcommands(clap::Command::new("repl"))
        .multicall(true)
        .try_get_matches_from(inputs)
        .and_then(|m| Command::from_arg_matches(&m));
    match c {
        Ok(cmd) => runtime.block_on(cmd.run(client, Path::new("."))),
        Err(e) => {
            e.print().unwrap();
            Ok(true)
        }
    }
}

fn main() -> Result<()> {
    let runtime = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to create runtime")?;
    let mut editor = Editor::with_history(
        rustyline::Config::builder()
            .auto_add_history(true)
            .max_history_size(1000)
            .unwrap()
            .build(),
        rustyline::history::MemHistory::new(),
    )
    .context("failed to create line editor")?;
    let client = api::Client::new();
    loop {
        match read_exec(&client, &runtime, &mut editor) {
            Ok(true) => (),
            Ok(false) => break,
            Err(e) => println!("error: {:?}", e),
        }
    }
    Ok(())
}
