use std::fmt::Display;

use codewars_api::User;
use codewars_types::rank::{Dan, Kyu, UserRankId};

pub fn show_user(u: &User) {
    println!("id: {}", u.id);
    println!("username: {}", u.username);
    if let Some(n) = &u.name {
        println!("name: {}", n);
    }
    println!("honor: {}", u.honor);
    if let Some(c) = &u.clan {
        println!("clan: {}", c);
    }
    if let Some(p) = u.leaderboard_position {
        println!("leaderboard position: {}", p);
    }

    if let Some(ss) = &u.skills {
        print!("skills:");
        ss.iter().for_each(|s| print!(" {}", s));
        println!();
    }

    fn show_rank<N: Display>(name: N, r: &codewars_api::UserRank) {
        use codewars_api::Color;
        println!(
            "{}: {}",
            name,
            yansi::Paint::new(&r.name).fg(match r.color {
                // black is invisible on terminal, use green instead
                Color::Black => yansi::Color::Green,
                Color::Blue => yansi::Color::Blue,
                Color::Purple => yansi::Color::Magenta,
                Color::Red => yansi::Color::Red,
                Color::White => yansi::Color::Default,
                Color::Yellow => yansi::Color::Yellow,
            })
        );
        println!("  score: {:}", r.score);

        let next_score = match r.rank {
            UserRankId::Kyu(k) => Some(match k {
                Kyu::Kyu8 => 20,
                Kyu::Kyu7 => 76,
                Kyu::Kyu6 => 229,
                Kyu::Kyu5 => 643,
                Kyu::Kyu4 => 1768,
                Kyu::Kyu3 => 4829,
                Kyu::Kyu2 => 13147,
                Kyu::Kyu1 => 35759,
            }),
            UserRankId::Dan(d) => match d {
                Dan::Dan1 => Some(97225),
                _ => None, // unknown score above 2 dan
            },
        };
        if let Some(next_score) = next_score {
            println!(
                "  next rank: {} {:.4}%",
                next_score,
                (r.score as f64 / next_score as f64) * 100.0
            );
        }
    }
    println!("----- ranks -----");
    show_rank("overall", &u.ranks.overall);
    for (k, v) in u.ranks.languages.iter() {
        show_rank(k.as_str(), v);
    }

    println!("---- challenges ----");
    println!("authored: {}", u.code_challenges.total_authored);
    println!("completed: {}", u.code_challenges.total_completed);
}
