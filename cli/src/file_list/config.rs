use eza::{
    fs::{
        dir_action::{DirAction, RecurseOptions},
        filter::{self, GitIgnore},
        DotFilter,
    },
    options,
    output::{color_scale, details, file_name, table, time, Mode, TerminalWidth, View},
    theme,
};

const COLOR_SCALE_OPT: color_scale::ColorScaleOptions = color_scale::ColorScaleOptions {
    mode: color_scale::ColorScaleMode::Gradient,
    min_luminance: 40,
    size: false,
    age: false,
};

#[inline]
pub fn theme_opt() -> theme::Options {
    theme::Options {
        use_colours: theme::UseColours::Automatic,
        colour_scale: COLOR_SCALE_OPT,
        definitions: theme::Definitions::default(),
    }
}

pub const DETAILS_OPT: details::Options = details::Options {
    table: Some({
        use table::*;
        Options {
            size_format: SizeFormat::BinaryBytes,
            time_format: time::TimeFormat::ISOFormat,
            user_format: UserFormat::Name,
            group_format: GroupFormat::Regular,
            flags_format: FlagsFormat::Long,
            columns: Columns {
                time_types: TimeTypes {
                    modified: true,
                    changed: false,
                    accessed: false,
                    created: true,
                },
                inode: false,
                links: false,
                blocksize: false,
                group: true,
                git: true,
                subdir_git_repos: false,
                subdir_git_repos_no_stat: false,
                octal: false,
                security_context: false,
                file_flags: true,
                permissions: true,
                filesize: true,
                user: true,
            },
        }
    }),
    header: true,
    xattr: false,
    secattr: false,
    mounts: false,
    color_scale: COLOR_SCALE_OPT,
};

#[inline]
pub const fn file_name_opt(is_a_tty: bool) -> file_name::Options {
    file_name::Options {
        classify: file_name::Classify::JustFilenames,
        show_icons: file_name::ShowIcons::Automatic(1),
        quote_style: file_name::QuoteStyle::NoQuotes,
        embed_hyperlinks: file_name::EmbedHyperlinks::Off,
        is_a_tty,
    }
}

pub const DOT_FILTER: DotFilter = DotFilter::Dotfiles;

pub const GIT_IGNORE: GitIgnore = GitIgnore::Off;

#[inline]
pub fn filter() -> filter::FileFilter {
    filter::FileFilter {
        list_dirs_first: false,
        sort_field: filter::SortField::Name(filter::SortCase::ABCabc),
        flags: Vec::new(),
        dot_filter: DOT_FILTER,
        ignore_patterns: filter::IgnorePatterns::empty(),
        git_ignore: GIT_IGNORE,
    }
}

pub const RECURSE_OPT: RecurseOptions = RecurseOptions {
    tree: true,
    max_depth: Some(5),
};

pub const GIT_REPO: bool = true;

pub const TERM_WIDTH: TerminalWidth = TerminalWidth::Automatic;

pub const DEREF_LINKS: bool = true;

pub const TOTAL_SIZE: bool = false;

#[allow(unused)]
pub fn options(is_a_tty: bool) -> options::Options {
    options::Options {
        dir_action: DirAction::Recurse(RECURSE_OPT),
        filter: filter(),
        view: View {
            mode: Mode::Details(DETAILS_OPT),
            width: TERM_WIDTH,
            file_style: file_name_opt(is_a_tty),
            deref_links: DEREF_LINKS,
            total_size: TOTAL_SIZE,
        },
        theme: theme_opt(),
        stdin: options::stdin::FilesInput::Args,
    }
}
