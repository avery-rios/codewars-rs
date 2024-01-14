use anyhow::{bail, Context, Result};
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::{
    collections::btree_map,
    fs,
    path::{Path, PathBuf},
};

use codewars_types::{KataId, KnownLangId};
use codewars_unofficial::project::{self, ProjectInfo, Session, SessionInfo};
use codewars_workspace::{self as workspace, WorkspaceObject};

use crate::{
    command::{next_cmd, print_err, CmdEnv, CmdState},
    kata,
};

#[derive(Debug, Serialize, Deserialize)]
struct SessionState {
    kata_id: KataId,
    language: KnownLangId,
    project: ProjectInfo,
    session: SessionInfo,
}

const SESSION_FILE: &str = "session.json";

fn create_workspace_dir(env: &CmdEnv, state: &SessionState, lang: &str) -> Result<PathBuf> {
    let mut dir = Path::new(&env.workspace).join(state.kata_id.to_string());
    dir.push(lang);
    fs::create_dir_all(&dir).context("failed to create workspace dir")?;

    dir.push(SESSION_FILE);
    fs::write(&dir, serde_json::to_string(&state).unwrap())
        .context("failed to write session state")?;
    dir.pop();

    Ok(dir)
}

pub fn start_session(
    env: &CmdEnv,
    cmd_state: &mut CmdState,
    kata: KataId,
    lang: KnownLangId,
) -> Result<()> {
    let client = env.unofficial_client.as_ref().context("login required")?;
    let project = env
        .runtime
        .block_on(client.start_project(&kata, lang))
        .context("failed to start project")?;
    let ses_state = SessionState {
        kata_id: kata,
        language: lang,
        session: env
            .runtime
            .block_on(project::start_session(client, &project))
            .context("failed to start session")?,
        project,
    };
    let session = Session::from_project(client, &ses_state.project, &ses_state.session);
    match lang {
        KnownLangId::Rust => {
            let ws = workspace::Rust::create(
                create_workspace_dir(env, &ses_state, "rust")?,
                &session.info.setup,
                &session.info.example_fixture,
            )
            .context("failed to create workspace")?;
            session_cmd(env, cmd_state, &ses_state.kata_id, lang, session, &ws)
        }
        l => {
            bail!("Unsupported language {}", l)
        }
    }
}

pub fn open_session(env: &CmdEnv, cmd_state: &mut CmdState, path: impl AsRef<Path>) -> Result<()> {
    let client = env.unofficial_client.as_ref().context("login required")?;
    let state: SessionState = serde_json::from_slice(
        &fs::read(path.as_ref().join(SESSION_FILE)).context("failed to read session file")?,
    )
    .context("invalid session file")?;
    let session = Session::from_project(client, &state.project, &state.session);
    match state.language {
        KnownLangId::Rust => session_cmd(
            env,
            cmd_state,
            &state.kata_id,
            state.language,
            session,
            &workspace::Rust::open(path.as_ref().to_path_buf()),
        ),
        l => {
            bail!("Unsupported language {}", l)
        }
    }
}

#[derive(Debug, Subcommand)]
enum SessionCmd {
    /// show session info
    Show,
    /// run sample test
    Test,
    Attempt,
    Submit,
    Clean,
    /// save solution code
    Save {
        #[arg(long)]
        tag: Option<String>,
    },
    /// back to last menu
    Back,
}

fn show_session(kata: &KataId, session: &Session<'_, '_, '_>) {
    let info = session.info;
    println!("Kata id: {}", kata);
    println!("Language: {} {}", info.language_name, info.active_version);
    println!("Test framework: {}", info.test_framework);
    println!("Solution id: {}", info.solution_id);
}

mod result;

fn clean(workspace: &dyn WorkspaceObject) -> Result<()> {
    workspace.clean().context("failed to clean workspace")?;
    let session_file = workspace.root().join(SESSION_FILE);
    fs::remove_file(session_file).context("failed to remove session file")
}

fn save(
    env: &CmdEnv,
    cmd_state: &mut CmdState,
    kata: &KataId,
    lang: KnownLangId,
    tag: Option<String>,
    workspace: &dyn WorkspaceObject,
) -> Result<()> {
    let mut kata_dir = match cmd_state.index.kata.entry(kata.to_owned()) {
        btree_map::Entry::Occupied(o) => Path::new(&env.root).join(&o.get().path),
        btree_map::Entry::Vacant(v) => {
            println!("Getting kata {}", kata);

            let info = env
                .runtime
                .block_on(env.api_client.get_challenge(kata))
                .context("failed to get kata")?;
            let dir_name = codewars_solution::kata_dir(kata, &info.slug);
            let path = Path::new(&env.root).join(&dir_name);
            fs::create_dir(&path).context("failed to create dir")?;
            let entry = codewars_solution::index::IndexEntry {
                name: info.name.clone(),
                slug: info.slug.clone(),
                path: dir_name,
            };
            kata::save_kata(
                path.clone(),
                &{
                    use codewars_solution::*;
                    Metadata {
                        version: Version::CURRENT,
                        api_version: ApiVersion::CURRENT,
                        created_at: chrono::Local::now().into(),
                        updated_at: Vec::new(),
                    }
                },
                info,
            )
            .context("failed to save kata info")?;
            v.insert(entry);
            path
        }
    };
    match tag {
        Some(t) => kata_dir.push(format!("{}-{}", lang, t)),
        None => kata_dir.push(lang.as_str()),
    }
    println!("Solution saved to {}", kata_dir.display());
    fs_extra::dir::copy(
        workspace.root(),
        kata_dir,
        &fs_extra::dir::CopyOptions::new()
            .overwrite(false)
            .copy_inside(true),
    )
    .context("failed to copy dir")?;
    Ok(())
}

fn session_cmd(
    env: &CmdEnv,
    state: &mut CmdState,
    kata: &KataId,
    lang: KnownLangId,
    session: Session<'_, '_, '_>,
    workspace: &dyn WorkspaceObject,
) -> Result<()> {
    let prompt = format!(
        "kata {} ({} {})> ",
        kata, &session.info.language_name, &session.info.active_version
    );
    loop {
        match next_cmd::<SessionCmd>(&prompt, &mut state.editor) {
            SessionCmd::Show => show_session(kata, &session),
            SessionCmd::Test => {
                match workspace
                    .get_code()
                    .context("failed to read code")
                    .and_then(|c| {
                        env.runtime
                            .block_on(session.test(&c.solution, &c.fixture))
                            .context("failed to run test")
                    }) {
                    Ok(r) => result::show("sample", &r),
                    Err(e) => print_err(e),
                }
            }
            SessionCmd::Attempt => {
                match workspace
                    .get_code()
                    .context("failed to read code")
                    .and_then(|c| {
                        env.runtime
                            .block_on(session.attempt(&c.solution, &c.fixture))
                            .context("failed to attempt test")
                    }) {
                    Ok(r) => result::show("full", &r),
                    Err(e) => print_err(e),
                }
            }
            SessionCmd::Submit => {
                if let Err(e) = env.runtime.block_on(session.submit()) {
                    print_err(anyhow::Error::new(e))
                }
            }
            SessionCmd::Clean => {
                if let Err(e) = clean(workspace) {
                    print_err(e)
                }
            }
            SessionCmd::Save { tag } => {
                if let Err(e) = save(env, state, kata, lang, tag, workspace) {
                    print_err(e)
                }
            }
            SessionCmd::Back => break,
        }
    }
    Ok(())
}
