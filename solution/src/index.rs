use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

use codewars_types::KataId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub name: String,
    pub slug: String,
    pub path: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Index {
    pub kata: BTreeMap<KataId, IndexEntry>,
}

#[derive(Debug, thiserror::Error)]
enum BuildErrorInner {
    #[error("failed to read dir")]
    OpenDir(#[source] io::Error),
    #[error("failed to read dir entry")]
    ReadDirEntry(#[source] io::Error),
    #[error("invalid path {}", path.display())]
    InvalidPath { path: PathBuf },
    #[error("failed to read info {}",path.display())]
    ReadInfo {
        #[source]
        source: io::Error,
        path: PathBuf,
    },
    #[error("failed to deserialize json {}",path.display())]
    DeserializeJson {
        #[source]
        source: serde_json::Error,
        path: PathBuf,
    },
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct BuildError(#[from] BuildErrorInner);

#[derive(Debug, thiserror::Error)]
enum OpenErrorInner {
    #[error("failed to read file")]
    ReadFile(#[source] io::Error),
    #[error("failed to deserialize json")]
    Json(#[source] serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct OpenError(#[from] OpenErrorInner);

pub const INDEX_FILE: &str = "index.json";

impl Index {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(root: impl AsRef<Path>) -> Result<Self, BuildError> {
        let mut ret = BTreeMap::new();
        for d in fs::read_dir(root).map_err(BuildErrorInner::OpenDir)? {
            let mut d = d.map_err(BuildErrorInner::ReadDirEntry)?.path();
            if !d.is_dir() {
                continue;
            }
            let name = d
                .file_name()
                .and_then(|f| f.to_str())
                .ok_or_else(|| BuildErrorInner::InvalidPath { path: d.clone() })?
                .to_string();

            d.push(super::INFO_FILE);
            let info: super::KataInfo =
                serde_json::from_slice(&fs::read(&d).map_err(|e| BuildErrorInner::ReadInfo {
                    source: e,
                    path: d.clone(),
                })?)
                .map_err(|e| BuildErrorInner::DeserializeJson {
                    source: e,
                    path: d.clone(),
                })?;

            ret.insert(
                info.id,
                IndexEntry {
                    name: info.name,
                    slug: info.slug,
                    path: name,
                },
            );
        }
        Ok(Index { kata: ret })
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, OpenError> {
        Ok(
            serde_json::from_slice(&fs::read(path).map_err(OpenErrorInner::ReadFile)?)
                .map_err(OpenErrorInner::Json)?,
        )
    }

    pub fn write(&self, path: impl AsRef<Path>) -> io::Result<()> {
        fs::write(path, serde_json::to_vec(self).unwrap())
    }
}
