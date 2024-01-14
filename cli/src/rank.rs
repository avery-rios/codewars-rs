use std::fmt::Display;
use yansi::Paint;

use codewars_types::rank::{KataRankId, Kyu};

#[derive(Debug)]
pub struct ShowKataRank(pub KataRankId);

impl Display for ShowKataRank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 .0 {
            Kyu::Kyu1 => Paint::magenta("1 kyu").fmt(f),
            Kyu::Kyu2 => Paint::magenta("2 kyu").fmt(f),
            Kyu::Kyu3 => Paint::blue("3 kyu").fmt(f),
            Kyu::Kyu4 => Paint::blue("4 kyu").fmt(f),
            Kyu::Kyu5 => Paint::yellow("5 kyu").fmt(f),
            Kyu::Kyu6 => Paint::yellow("6 kyu").fmt(f),
            Kyu::Kyu7 => f.write_str("7 kyu"),
            Kyu::Kyu8 => f.write_str("8 kyu"),
        }
    }
}
