use serde::{Deserialize, Serialize};

pub mod lang;
pub use lang::{KnownLangId, LangId};

/// Codewars api version
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ApiVersion {
    #[serde(rename = "v1")]
    V1,
}
impl ApiVersion {
    pub const CURRENT: Self = Self::V1;
}

pub mod rank;
pub use rank::{KataRankId, UserRankId};

pub mod kata_id;
pub use kata_id::KataId;
