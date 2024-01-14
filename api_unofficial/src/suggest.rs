use reqwest::header::AUTHORIZATION;
use serde::{de, Deserialize, Deserializer};

use codewars_types::{KataId, KataRankId, KnownLangId, LangId};

use crate::{jwt::GetJwtError, Client};

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum SuggestStrategy {
    #[serde(rename = "reference_workout")]
    Fundamental,
    #[serde(rename = "default")]
    RankUp,
    #[serde(rename = "retrain_workout")]
    Practice,
    #[serde(rename = "beta_workout")]
    Beta,
    #[serde(rename = "random")]
    Random,
}
impl SuggestStrategy {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Fundamental => "reference_workout",
            Self::RankUp => "default",
            Self::Practice => "retrain_workout",
            Self::Beta => "beta_workout",
            Self::Random => "random",
        }
    }
}

fn deserialize_opt_kata_rank<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<KataRankId>, D::Error> {
    struct Visitor;
    impl<'de> de::Visitor<'de> for Visitor {
        type Value = Option<KataRankId>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an integer between -8 and -1")
        }
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match v.try_into().ok().and_then(KataRankId::from_id) {
                Some(v) => Ok(Some(v)),
                None => Err(E::invalid_value(de::Unexpected::Signed(v), &self)),
            }
        }
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }
    deserializer.deserialize_i64(Visitor)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestedKata {
    pub success: bool,
    pub strategy: SuggestStrategy,
    pub language: LangId,
    pub id: KataId,
    pub name: String,
    pub description: String,
    pub system_tags: Vec<String>,
    #[serde(deserialize_with = "deserialize_opt_kata_rank")]
    pub rank: Option<KataRankId>,
    pub href: String,
}

pub struct Suggest<'c> {
    client: &'c reqwest::Client,
    jwt: String,
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct StartSuggestError(#[from] GetJwtError);

impl Client {
    pub async fn suggest_kata(&self) -> Result<Suggest<'_>, StartSuggestError> {
        let jwt = self.get_jwt("https://www.codewars.com/dashboard").await?;
        Ok(Suggest {
            client: &self.client,
            jwt,
        })
    }
}

impl<'c> Suggest<'c> {
    pub async fn suggest(
        &self,
        lang: KnownLangId,
        strategy: SuggestStrategy,
        skip: bool,
    ) -> reqwest::Result<SuggestedKata> {
        self.client
            .get(format!(
                "https://www.codewars.com/trainer/peek/{}/{}?dequeue={}",
                lang.as_str(),
                strategy.as_str(),
                skip
            ))
            .header(AUTHORIZATION, &self.jwt)
            .send()
            .await?
            .json()
            .await
    }
}
