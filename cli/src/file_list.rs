use anyhow::{bail, Context, Result};
use eza::{
    fs::{dir_action::DirAction, filter::GitIgnore, File},
    options::{self, OptionsError, OptionsResult},
    output::{details, grid, grid_details, lines, Mode},
    theme,
};
use std::{
    error,
    ffi::OsString,
    fmt::Display,
    io::{self, IsTerminal},
    path::PathBuf,
};

struct Var;
impl options::Vars for Var {
    fn get(&self, name: &'static str) -> Option<OsString> {
        std::env::var_os(name)
    }
}

mod config;

#[derive(Debug)]
struct ParseErr(OptionsError);
impl Display for ParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl error::Error for ParseErr {}

enum EzaOptions {
    Default,
    Custom {
        console_width: usize,
        opt: options::Options,
    },
}
pub struct Options {
    theme: theme::Theme,
    is_a_tty: bool,
    options: EzaOptions,
}
impl Options {
    pub fn parse(args: &str) -> Result<Self> {
        let opts = shlex::split(args)
            .context("failed to split args")?
            .iter()
            .map(OsString::from)
            .collect::<Vec<_>>();
        let is_a_tty = io::stdout().is_terminal();

        match options::Options::parse(opts.iter().map(OsString::as_os_str), &Var) {
            OptionsResult::Ok(opt, _) => Ok(Self {
                theme: opt.theme.to_theme(is_a_tty),
                is_a_tty,
                options: EzaOptions::Custom {
                    console_width: opt
                        .view
                        .width
                        .actual_terminal_width()
                        .context("failed to get terminal width")?,
                    opt,
                },
            }),
            OptionsResult::InvalidOptions(err) => {
                Err(anyhow::Error::new(ParseErr(err)).context("invalid options"))
            }
            OptionsResult::Help(_) => bail!("Unexpected help option"),
            OptionsResult::Version(_) => bail!("Unexpected version option"),
        }
    }
    pub fn default() -> Self {
        let is_a_tty = io::stdout().is_terminal();
        Self {
            theme: config::theme_opt().to_theme(is_a_tty),
            is_a_tty,
            options: EzaOptions::Default,
        }
    }
}

const fn git_ignore_to_bool(gi: GitIgnore) -> bool {
    match gi {
        GitIgnore::CheckAndIgnore => true,
        GitIgnore::Off => false,
    }
}

fn list_dir_custom(
    o: &Options,
    console_width: usize,
    opt: &options::Options,
    path: PathBuf,
) -> Result<()> {
    let files =
        Vec::from([
            File::from_args(path, None, None, opt.view.deref_links, opt.view.total_size)
                .context("failed to open file")?,
        ]);
    let out = &mut io::stdout();
    let theme = &o.theme;
    let git_ignoring = git_ignore_to_bool(opt.filter.git_ignore);

    match &opt.view.mode {
        Mode::Grid(go) => grid::Render {
            files,
            theme,
            file_style: &opt.view.file_style,
            opts: go,
            console_width,
            filter: &opt.filter,
        }
        .render(out),
        Mode::Details(d_opts) => details::Render {
            dir: None,
            files,
            theme,
            file_style: &opt.view.file_style,
            opts: d_opts,
            recurse: if let DirAction::Recurse(r) = opt.dir_action {
                Some(r)
            } else {
                None
            },
            filter: &opt.filter,
            git_ignoring,
            git: None,
            git_repos: false,
        }
        .render(out),
        Mode::GridDetails(go) => grid_details::Render {
            dir: None,
            files,
            theme,
            file_style: &opt.view.file_style,
            grid: &go.grid,
            details: &go.details,
            filter: &opt.filter,
            row_threshold: go.row_threshold,
            git_ignoring,
            git: None,
            console_width,
            git_repos: false,
        }
        .render(out),
        Mode::Lines => lines::Render {
            files,
            theme,
            file_style: &opt.view.file_style,
            filter: &opt.filter,
        }
        .render(out),
    }
    .context("failed to render output")
}

fn list_dir_def(o: &Options, path: PathBuf) -> Result<()> {
    details::Render {
        dir: None,
        files: Vec::from([File::from_args(
            path,
            None,
            None,
            config::DEREF_LINKS,
            config::TOTAL_SIZE,
        )
        .context("failed to open file")?]),
        theme: &o.theme,
        file_style: &config::file_name_opt(o.is_a_tty),
        opts: &config::DETAILS_OPT,
        recurse: Some(config::RECURSE_OPT),
        filter: &config::filter(),
        git_ignoring: git_ignore_to_bool(config::GIT_IGNORE),
        git: None,
        git_repos: config::GIT_REPO,
    }
    .render(&mut io::stdout())
    .context("failed to render file list")
}

pub fn list_dir(o: &Options, path: PathBuf) -> Result<()> {
    match &o.options {
        EzaOptions::Default => list_dir_def(o, path),
        EzaOptions::Custom { console_width, opt } => list_dir_custom(o, *console_width, opt, path),
    }
}
