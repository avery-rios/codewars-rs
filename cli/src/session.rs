use anyhow::{bail, Context, Result};
use clap::{Args, Subcommand};
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
    command::{new_editor, next_cmd, print_err, CmdEnv, CmdState},
    file_list, kata,
};

#[derive(Debug, Serialize, Deserialize)]
struct SessionState {
    kata_id: KataId,
    language: KnownLangId,
    slug: String,
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

fn get_kata(env: &CmdEnv, cmd_state: &mut CmdState, kata: &KataId) -> Result<String> {
    match cmd_state.index_mut().kata.entry(kata.to_owned()) {
        btree_map::Entry::Occupied(o) => Ok(o.get().slug.clone()),
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
            let slug = info.slug.clone();
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
            cmd_state.index_dirty = true;
            Ok(slug)
        }
    }
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
        kata_id: kata.clone(),
        language: lang,
        slug: get_kata(env, cmd_state, &kata).context("failed to get kata")?,
        session: {
            let mut info = env
                .runtime
                .block_on(project::start_session(client, &project))
                .context("failed to start session")?;
            let version_id =
                dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .with_prompt("Language version?")
                    .items(
                        &info
                            .language_versions
                            .iter()
                            .map(|v| v.label.as_str())
                            .collect::<Vec<_>>(),
                    )
                    .interact()
                    .context("failed to get language version")?;
            info.active_version = info.language_versions[version_id].id.clone();
            info
        },
        project,
    };
    let workspace_root = create_workspace_dir(env, &ses_state, lang.as_str())
        .context("failed to create workspace dir")?;
    let workspace_cfg = workspace::Config {
        slug: &ses_state.slug,
        version_id: &ses_state.session.active_version,
        code: &ses_state.session.setup,
        fixture: &ses_state.session.example_fixture,
        has_preload: dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Has preloaded code?")
            .default(false)
            .show_default(true)
            .interact()
            .context("failed to prompt if there is preloaded code")?,
    };
    match lang {
        KnownLangId::Coq => session_cmd(
            env,
            &ses_state,
            &workspace_root,
            &workspace::Coq::create(&workspace_root, workspace_cfg)
                .context("failed to create workspace")?,
        ),
        KnownLangId::Rust => session_cmd(
            env,
            &ses_state,
            &workspace_root,
            &workspace::Rust::create(&workspace_root, workspace_cfg)
                .context("failed to create workspace")?,
        ),
        KnownLangId::Haskell => session_cmd(
            env,
            &ses_state,
            &workspace_root,
            &workspace::Haskell::create(&workspace_root, workspace_cfg)
                .context("failed to create workspace")?,
        ),
        l => {
            bail!("Unsupported language {l}")
        }
    }
}

pub fn open_session(env: &CmdEnv, _: &mut CmdState, path: impl AsRef<Path>) -> Result<()> {
    let state: SessionState = serde_json::from_slice(
        &fs::read(path.as_ref().join(SESSION_FILE)).context("failed to read session file")?,
    )
    .context("invalid session file")?;
    let workspace_root = path.as_ref();
    match state.language {
        KnownLangId::Coq => session_cmd(
            env,
            &state,
            workspace_root,
            &workspace::Coq::open(workspace_root).context("failed to open workspace")?,
        ),
        KnownLangId::Rust => session_cmd(
            env,
            &state,
            workspace_root,
            &workspace::Rust::open(workspace_root).context("failed to open workspace")?,
        ),
        KnownLangId::Haskell => session_cmd(
            env,
            &state,
            workspace_root,
            &workspace::Haskell::open(workspace_root).context("failed to open workspace")?,
        ),
        l => {
            bail!("Unsupported language {l}")
        }
    }
}

#[derive(Debug, Args)]
struct SaveOpt {
    /// no list files
    #[arg(long)]
    no_list: bool,
    #[arg(long, short = 'y')]
    yes: bool,
    #[arg(long)]
    tag: Option<String>,
}

#[derive(Debug, Clone, Copy, Subcommand)]
enum CleanCmd {
    Build,
    Session,
    All,
}

#[derive(Debug, Subcommand)]
enum SessionCmd {
    /// show session info
    Show,
    /// run sample test
    Test,
    Attempt,
    Submit,
    Clean {
        #[command(subcommand)]
        cmd: CleanCmd,
    },
    /// save solution code
    Save(SaveOpt),
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

fn clean(cmd: CleanCmd, root: &Path, workspace: &dyn WorkspaceObject) -> Result<()> {
    fn clean_session(root: &Path, workspace: &dyn WorkspaceObject) -> Result<()> {
        workspace
            .clean_session()
            .context("failed to clean session workspace")?;
        let session_file = root.join(SESSION_FILE);
        fs::remove_file(session_file).context("failed to remove session file")
    }

    match cmd {
        CleanCmd::Build => workspace.clean_build().context("failed to clean build"),
        CleanCmd::Session => clean_session(root, workspace),
        CleanCmd::All => {
            workspace
                .clean_build()
                .context("failed to clean build result")?;
            clean_session(root, workspace)
        }
    }
}

fn save(env: &CmdEnv, ses_state: &SessionState, opt: SaveOpt, root: &Path) -> Result<()> {
    let mut kata_dir = Path::new(&env.root).join(codewars_solution::kata_dir(
        &ses_state.kata_id,
        &ses_state.slug,
    ));
    match opt.tag {
        Some(t) => kata_dir.push(format!("{}-{}", ses_state.language, t)),
        None => kata_dir.push(ses_state.language.as_str()),
    }
    println!("Solution will be saved to {}", kata_dir.display());

    if !opt.no_list {
        println!("Files will be saved:");
        file_list::list_dir(&env.list_option, root.to_path_buf())
            .context("failed to list workspace dir")?;
    }
    if !(opt.yes
        || dialoguer::Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Save solution? ")
            .interact()
            .context("failed to read select")?)
    {
        println!("Canceled solution saving");
        return Ok(());
    }

    fs_extra::dir::copy(
        root,
        kata_dir,
        &fs_extra::dir::CopyOptions::new()
            .overwrite(false)
            .copy_inside(true),
    )
    .context("failed to copy dir")?;
    println!("Solution saved");
    Ok(())
}

fn session_cmd(
    env: &CmdEnv,
    ses_state: &SessionState,
    workspace_root: &Path,
    workspace: &dyn WorkspaceObject,
) -> Result<()> {
    let prompt = format!(
        "kata {}[{}] ({} {})> ",
        ses_state.slug,
        ses_state.kata_id,
        &ses_state.session.language_name,
        &ses_state.session.active_version
    );
    let mut editor = new_editor().context("failed to create editor")?;
    let session = Session::from_project(
        env.unofficial_client
            .as_ref()
            .context("login is required")?,
        &ses_state.project,
        &ses_state.session,
    );
    loop {
        match next_cmd::<SessionCmd>(&prompt, &mut editor) {
            SessionCmd::Show => show_session(&ses_state.kata_id, &session),
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
            SessionCmd::Clean { cmd } => {
                if let Err(e) = clean(cmd, workspace_root, workspace) {
                    print_err(e)
                }
            }
            SessionCmd::Save(opt) => {
                if let Err(e) = save(env, ses_state, opt, workspace_root) {
                    print_err(e)
                }
            }
            SessionCmd::Back => break,
        }
    }
    Ok(())
}
