use anyhow::{Context, Result};

use codewars_types::KnownLangId;
use codewars_unofficial::suggest::{Suggest, SuggestStrategy, SuggestedKata};

use crate::{
    command::{next_cmd, print_err, CmdEnv, CmdState},
    rank::ShowKataRank,
    session,
};

pub fn start_suggest(env: &CmdEnv, state: &mut CmdState, lang: KnownLangId) -> Result<()> {
    let client = env.unofficial_client.as_ref().context("Login required")?;
    suggest_cmd(
        env,
        state,
        env.runtime
            .block_on(client.suggest_kata())
            .context("failed to start suggestion")?,
        lang,
        SuggestStrategy::RankUp,
    )
}

fn to_prompt(lang: KnownLangId, strategy: SuggestStrategy) -> String {
    format!(
        "suggest@{} ({})> ",
        lang,
        match strategy {
            SuggestStrategy::Fundamental => "fundamental",
            SuggestStrategy::RankUp => "rank up",
            SuggestStrategy::Practice => "practice",
            SuggestStrategy::Random => "random",
            SuggestStrategy::Beta => "beta",
        }
    )
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum Strategy {
    Fundamental,
    RankUp,
    Practice,
    Random,
    Beta,
}

#[derive(Debug, clap::Subcommand)]
enum SuggestCmd {
    Next {
        #[arg(long)]
        skip: bool,
    },
    Config {
        #[arg(long)]
        lang: Option<KnownLangId>,
        #[arg(long)]
        strategy: Option<Strategy>,
    },
    Show,
    Train,
    Back,
}

fn show_suggestion(suggestion: &SuggestedKata) {
    println!("Name: {}", suggestion.name);
    if let Some(r) = suggestion.rank {
        println!("Rank: {}", ShowKataRank(r));
    }
    println!("Id: {}", suggestion.id);
    println!("Url: {}", suggestion.href);
    if !suggestion.system_tags.is_empty() {
        print!("Tags:");
        for t in &suggestion.system_tags {
            print!(" [{}]", t);
        }
        println!()
    }
}

fn suggest_cmd(
    env: &CmdEnv,
    state: &mut CmdState,
    suggest: Suggest<'_>,
    mut lang: KnownLangId,
    mut strategy: SuggestStrategy,
) -> Result<()> {
    let mut prompt = to_prompt(lang, strategy);
    let mut current = env
        .runtime
        .block_on(suggest.suggest(lang, strategy, false))
        .context("failed to get suggestion")?;
    show_suggestion(&current);
    loop {
        match next_cmd::<SuggestCmd>(&prompt, &mut state.editor) {
            SuggestCmd::Next { skip } => {
                match env.runtime.block_on(suggest.suggest(lang, strategy, skip)) {
                    Ok(n) => {
                        current = n;
                        show_suggestion(&current);
                    }
                    Err(e) => print_err(e.into()),
                }
            }
            SuggestCmd::Config {
                lang: opt_lang,
                strategy: opt_str,
            } => {
                if let Some(l) = opt_lang {
                    lang = l;
                }
                if let Some(s) = opt_str {
                    strategy = match s {
                        Strategy::Fundamental => SuggestStrategy::Fundamental,
                        Strategy::RankUp => SuggestStrategy::RankUp,
                        Strategy::Practice => SuggestStrategy::Practice,
                        Strategy::Beta => SuggestStrategy::Beta,
                        Strategy::Random => SuggestStrategy::Random,
                    };
                }
                prompt = to_prompt(lang, strategy);
            }
            SuggestCmd::Show => show_suggestion(&current),
            SuggestCmd::Train => {
                if let Err(e) = session::start_session(env, state, current.id.clone(), lang) {
                    print_err(e)
                }
            }
            SuggestCmd::Back => break,
        }
    }
    Ok(())
}
