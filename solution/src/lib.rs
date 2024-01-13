use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub use codewars_types::{rank, ApiVersion, KataId};
use rank::KataRankId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub username: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KataApprove {
    pub rank: KataRankId,
    pub approver: Author,
    pub approved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KataInfo {
    pub name: String,
    pub id: KataId,
    pub slug: String,
    pub url: String,
    pub created_by: Author,
    pub created_at: DateTime<Utc>,
    pub approve: Option<KataApprove>,
    pub category: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version(pub u8, pub u8);
impl Version {
    pub const CURRENT: Self = Self(0, 1);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub version: Version,
    pub api_version: ApiVersion,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: Vec<DateTime<FixedOffset>>,
}

/// push kata path to a [PathBuf]
/// Slug will be truncated if it's too long
pub fn kata_path(root: impl AsRef<Path>, kata: &KataInfo) -> PathBuf {
    const MAX_FILENAME: usize = 128;
    const ID_LEN: usize = 24 + 2; // length of id and brackets
    let slug = if kata.slug.len() + ID_LEN > MAX_FILENAME {
        &kata.slug[0..(MAX_FILENAME - ID_LEN)]
    } else {
        kata.slug.as_str()
    };
    root.as_ref().join(format!("{}[{}]", slug, &kata.id))
}

/// save kata info under directory `root`
/// - metadata file meta.json
/// - kata info file info.json
/// - kata description description.md
/// - store solution under directory named by language,
///   may be with a tag separated by dash. Like `haskell-tag1`
pub fn write_kata(
    mut root: PathBuf,
    meta: &Metadata,
    info: &KataInfo,
    desc: &str,
) -> io::Result<()> {
    use serde_json::to_vec_pretty;
    root.push("meta.json");
    fs::write(&root, to_vec_pretty(meta).unwrap())?;
    root.pop();

    root.push("info.json");
    fs::write(&root, to_vec_pretty(info).unwrap())?;
    root.pop();

    root.push("description.md");
    fs::write(&root, desc)
}
