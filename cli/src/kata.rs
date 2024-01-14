use anyhow::{Context, Result};
use codewars_types::KataId;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn save_kata(
    root: PathBuf,
    meta: &codewars_solution::Metadata,
    kata: codewars_api::CodeChallenge,
) -> Result<()> {
    use codewars_solution::*;

    fn to_author(auth: api::Author) -> Author {
        Author {
            username: auth.username,
            url: auth.url,
        }
    }
    fn to_approve(
        rank: Option<api::KataRank>,
        approver: Option<api::Author>,
        approved_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Option<KataApprove> {
        Some(KataApprove {
            rank: rank?.id,
            approver: match approver {
                Some(a) => to_author(a),
                None => return None,
            },
            approved_at: approved_at?,
        })
    }

    write_kata(
        root,
        meta,
        &KataInfo {
            name: kata.name,
            id: kata.id,
            slug: kata.slug,
            url: kata.url,
            approve: to_approve(kata.rank, kata.approved_by, kata.approved_at),
            created_by: to_author(kata.created_by),
            created_at: kata.published_at,
            category: kata.category,
            tags: kata.tags,
        },
        &kata.description,
    )
    .context("failed to write kata")
}

pub async fn get_kata(id: &KataId, client: &codewars_api::Client, root: &Path) -> Result<()> {
    use codewars_solution::{kata_dir, ApiVersion, Metadata, Version};

    let kata = client
        .get_challenge(id)
        .await
        .context("failed to get kata info")?;
    let dir_name = kata_dir(id, &kata.slug);
    let kata_root = root.join(&dir_name);
    if kata_root.exists() {
        anyhow::bail!("Kata {} is already fetched", id);
    }
    fs::create_dir(&kata_root).context("failed to create kata dir")?;
    println!("Kata {} will be saved to {}", id, kata_root.display());
    save_kata(
        kata_root,
        &Metadata {
            version: Version::CURRENT,
            api_version: ApiVersion::CURRENT,
            created_at: chrono::Local::now().fixed_offset(),
            updated_at: Vec::new(),
        },
        kata,
    )
    .context("failed to write kata")?;
    Ok(())
}
