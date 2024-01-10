use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Kyu {
    Kyu8,
    Kyu7,
    Kyu6,
    Kyu5,
    Kyu4,
    Kyu3,
    Kyu2,
    Kyu1,
}
impl Kyu {
    pub fn from_id(id: i8) -> Option<Self> {
        match id {
            -8 => Some(Self::Kyu8),
            -7 => Some(Self::Kyu7),
            -6 => Some(Self::Kyu6),
            -5 => Some(Self::Kyu5),
            -4 => Some(Self::Kyu4),
            -3 => Some(Self::Kyu3),
            -2 => Some(Self::Kyu2),
            -1 => Some(Self::Kyu1),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Dan {
    Dan1,
    Dan2,
    Dan3,
    Dan4,
    Dan5,
    Dan6,
    Dan7,
    Dan8,
}
impl Dan {
    pub fn from_id(id: i8) -> Option<Self> {
        match id {
            1 => Some(Self::Dan1),
            2 => Some(Self::Dan2),
            3 => Some(Self::Dan3),
            4 => Some(Self::Dan4),
            5 => Some(Self::Dan5),
            6 => Some(Self::Dan6),
            7 => Some(Self::Dan7),
            8 => Some(Self::Dan8),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UserRankId {
    Kyu(Kyu),
    Dan(Dan),
}
impl UserRankId {
    pub fn from_id(id: i8) -> Option<Self> {
        if id < 0 {
            Kyu::from_id(id).map(Self::Kyu)
        } else {
            Dan::from_id(id).map(Self::Dan)
        }
    }
}

/// Kata rank. Only use 8kyu-1kyu
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct KataRankId(pub Kyu);
impl KataRankId {
    pub fn from_id(id: i8) -> Option<Self> {
        Kyu::from_id(id).map(Self)
    }
}
