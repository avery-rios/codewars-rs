//! # Codewars api v1
//! [codewars api docs](https://dev.codewars.com)
pub use codewars_types::{lang, rank};

use chrono::{DateTime, Utc};
use codewars_types::{
    kata_id::KataId,
    lang::LangId,
    rank::{KataRankId, UserRankId},
};
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    White,
    Yellow,
    Blue,
    Purple,
    Black,
    Red,
}

fn deserialize_user_rank_id<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<UserRankId, D::Error> {
    struct Visitor;
    impl<'de> de::Visitor<'de> for Visitor {
        type Value = UserRankId;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an integer between -8 and -1 or 1 and 8")
        }
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            v.try_into()
                .ok()
                .and_then(UserRankId::from_id)
                .ok_or_else(|| E::invalid_value(de::Unexpected::Signed(v), &self))
        }
    }
    deserializer.deserialize_i8(Visitor)
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserRank {
    #[serde(deserialize_with = "deserialize_user_rank_id")]
    pub rank: UserRankId,
    pub name: String,
    pub color: Color,
    pub score: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Ranks {
    pub overall: UserRank,
    pub languages: HashMap<LangId, UserRank>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeChallenges {
    pub total_authored: u32,
    pub total_completed: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub username: String,
    pub name: String,
    pub honor: u32,
    pub clan: String,
    pub leaderboard_position: u32,
    pub skills: Vec<String>,
    pub ranks: Ranks,
    pub code_challenges: CodeChallenges,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletedChallenge {
    pub id: KataId,
    pub name: String,
    pub slug: String,
    pub completed_at: DateTime<Utc>,
    pub completed_languages: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Paged<T> {
    pub total_pages: u32,
    pub total_items: u32,
    pub data: Vec<T>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthoredChallenge {
    pub id: KataId,
    pub name: String,
    pub description: String,
    pub rank: Option<i8>,
    pub rank_name: Option<String>,
    pub tags: Vec<String>,
    pub languages: Vec<LangId>,
}

fn deserialize_kata_rank_id<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<KataRankId, D::Error> {
    struct Visitor;
    impl<'de> de::Visitor<'de> for Visitor {
        type Value = KataRankId;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an integer between -8 and -1")
        }
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            v.try_into()
                .ok()
                .and_then(KataRankId::from_id)
                .ok_or_else(|| E::invalid_value(de::Unexpected::Signed(v), &self))
        }
    }
    deserializer.deserialize_i8(Visitor)
}

#[derive(Debug, Clone, Deserialize)]
pub struct KataRank {
    #[serde(deserialize_with = "deserialize_kata_rank_id")]
    pub id: KataRankId,
    pub name: String,
    pub color: Color,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Author {
    pub username: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Unresolved {
    pub issues: u32,
    pub suggestions: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeChallenge {
    pub id: KataId,
    pub name: String,
    pub slug: String,
    pub url: String,
    pub category: String,
    pub description: String,
    pub tags: Vec<String>,
    pub languages: Vec<LangId>,
    pub rank: Option<KataRank>,
    pub created_by: Author,
    pub published_at: DateTime<Utc>,
    pub approved_by: Option<Author>,
    pub approved_at: Option<DateTime<Utc>>,
    pub total_completed: u32,
    pub total_attempts: u32,
    pub total_stars: u32,
    pub vote_score: i32,
    pub contributors_wanted: bool,
    pub unresolved: Unresolved,
}

#[derive(Debug)]
pub struct Client {
    client: reqwest::Client,
}
impl Client {
    pub fn new() -> Self {
        Client {
            client: reqwest::Client::new(),
        }
    }
    async fn get<U: reqwest::IntoUrl, T: serde::de::DeserializeOwned>(
        &self,
        url: U,
    ) -> reqwest::Result<T> {
        self.client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
    pub async fn get_user(&self, usr: &str) -> reqwest::Result<User> {
        self.get(format!("https://www.codewars.com/api/v1/users/{}", usr))
            .await
    }
    pub async fn list_completed(
        &self,
        usr: &str,
        page: u32,
    ) -> reqwest::Result<Paged<CompletedChallenge>> {
        self.get(format!(
            "https://www.codewars.com/api/v1/users/{}/code-challenges/completed?page={}",
            usr, page
        ))
        .await
    }
    pub async fn list_authored(&self, usr: &str) -> reqwest::Result<Vec<AuthoredChallenge>> {
        #[derive(Deserialize)]
        struct Wrapper {
            data: Vec<AuthoredChallenge>,
        }
        self.get(format!(
            "https://www.codewars.com/api/v1/users/{}/code-challenges/authored",
            usr
        ))
        .await
        .map(|w: Wrapper| w.data)
    }
    pub async fn get_challenge(&self, id: &KataId) -> reqwest::Result<CodeChallenge> {
        self.get(format!(
            "https://www.codewars.com/api/v1/code-challenges/{}",
            id
        ))
        .await
    }
}
impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
