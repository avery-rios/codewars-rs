use anyhow::Context;
use clap::{FromArgMatches, Subcommand};
use rustyline::Editor;
use std::path::PathBuf;

use crate::file_list;

pub type LineEditor = Editor<(), rustyline::history::MemHistory>;

pub fn new_editor() -> rustyline::Result<LineEditor> {
    Editor::with_history(
        rustyline::Config::builder()
            .auto_add_history(true)
            .max_history_size(1000)
            .unwrap()
            .build(),
        rustyline::history::MemHistory::new(),
    )
}

pub fn next_cmd<C: FromArgMatches + Subcommand>(prompt: &str, editor: &mut LineEditor) -> C {
    loop {
        let inputs = match editor
            .readline(prompt)
            .map_err(anyhow::Error::new)
            .and_then(|i| shlex::split(&i).context("failed to split input"))
        {
            Ok(i) => i,
            Err(e) => {
                println!("error: {:?}", e);
                continue;
            }
        };
        let c = C::augment_subcommands(clap::Command::new("repl"))
            .multicall(true)
            .try_get_matches_from(inputs)
            .and_then(|m| C::from_arg_matches(&m));
        match c {
            Ok(cmd) => return cmd,
            Err(e) => e.print().unwrap(),
        }
    }
}

pub struct CmdEnv {
    pub root: String,
    pub index_path: PathBuf,
    pub workspace: String,
    pub runtime: tokio::runtime::Runtime,
    pub api_client: codewars_api::Client,
    pub unofficial_client: Option<codewars_unofficial::Client>,
    pub list_option: file_list::Options,
}

pub struct CmdState {
    pub editor: LineEditor,
    index: codewars_solution::index::Index,
    pub index_dirty: bool,
}
impl CmdState {
    pub fn new(editor: LineEditor, index: codewars_solution::index::Index) -> Self {
        Self {
            editor,
            index,
            index_dirty: false,
        }
    }
    pub fn index(&self) -> &codewars_solution::index::Index {
        &self.index
    }
    pub fn index_mut(&mut self) -> &mut codewars_solution::index::Index {
        self.index_dirty = true;
        &mut self.index
    }
}

pub fn print_err(err: anyhow::Error) {
    eprintln!("{}: {:?}", yansi::Paint::red("error"), err)
}
