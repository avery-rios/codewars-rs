use std::fmt::Display;

use codewars_types::{KataId, KnownLangId};
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};

use crate::{jwt::find_jwt, Client};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    id: String,
    jwt: String,
    lang: KnownLangId,
}

#[derive(Debug, thiserror::Error)]
pub enum StartProjectError {
    #[error("Error sending http request")]
    Http(
        #[from]
        #[source]
        reqwest::Error,
    ),
    #[error("Project id not found")]
    ProjectIdNotFound,
    #[error("Jwt not found")]
    JwtNotFound,
}

fn find_project_id(input: &str) -> Option<&str> {
    const FIND_STR: &str = "/api/v1/code-challenges/projects/";
    Some(
        input[input.find(FIND_STR)? + FIND_STR.len()..]
            .split_once('/')?
            .0,
    )
}

impl Client {
    pub async fn start_project(
        &self,
        kata: &KataId,
        lang: KnownLangId,
    ) -> Result<ProjectInfo, StartProjectError> {
        let resp = self
            .client
            .get(format!(
                "https://www.codewars.com/kata/{}/train/{}",
                kata,
                lang.as_str()
            ))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        log::trace!("start project body: {}", resp);
        let ret = ProjectInfo {
            id: String::from(find_project_id(&resp).ok_or(StartProjectError::ProjectIdNotFound)?),
            jwt: find_jwt(&resp)
                .ok_or(StartProjectError::JwtNotFound)?
                .to_string(),
            lang,
        };
        log::debug!("Start project {:?}", ret);
        Ok(ret)
    }
}

pub async fn start_session(
    client: &Client,
    info: &ProjectInfo,
) -> Result<SessionInfo, reqwest::Error> {
    client
        .post_with_csrf(format!(
            "https://www.codewars.com/kata/projects/{}/{}/session",
            &info.id, info.lang
        ))
        .header(AUTHORIZATION, &info.jwt)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangVersion {
    pub id: String,
    pub label: String,
    pub supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub setup: String,
    pub example_fixture: String,
    pub fixture: String,
    pub language_name: String,
    pub language_versions: Vec<LangVersion>,
    pub active_version: String,
    pub solution_id: String,
    pub package: String,
    pub test_framework: String,
}

#[derive(Debug, Clone, Copy)]
enum SessionErrKind {
    AuthRunner,
    RunTest,
    Notify,
}
#[derive(Debug)]
pub struct SessionError {
    kind: SessionErrKind,
    inner: reqwest::Error,
}
impl Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.kind {
            SessionErrKind::AuthRunner => "failed to authorize runner",
            SessionErrKind::RunTest => "failed to run test",
            SessionErrKind::Notify => "failed to notify code",
        })
    }
}
impl std::error::Error for SessionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}

pub mod result;
use result::TestResult;

pub struct Session<'c, 'p, 'i> {
    client: &'c Client,
    project: &'p ProjectInfo,
    pub info: &'i SessionInfo,
}
impl<'c, 'p, 'i> Session<'c, 'p, 'i> {
    pub fn from_project(
        client: &'c Client,
        project: &'p ProjectInfo,
        info: &'i SessionInfo,
    ) -> Self {
        Self {
            client,
            project,
            info,
        }
    }
    async fn auth_runner(&self) -> Result<String, reqwest::Error> {
        #[derive(Deserialize)]
        struct Auth {
            token: String,
        }
        let ret: Auth = self
            .client
            .post_with_csrf("https://www.codewars.com/api/v1/runner/authorize")
            .header(AUTHORIZATION, &self.project.jwt)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        log::debug!("Authorized runner with token: {}", &ret.token);
        Ok(ret.token)
    }
    async fn run_test(
        &self,
        token: &str,
        code: &str,
        fixture_ciphered: bool,
        fixture: &str,
    ) -> Result<TestResult, reqwest::Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct RunPayload<'a> {
            ciphered: &'a [&'a str],
            code: &'a str,
            fixture: &'a str,
            setup: &'a str,
            language: &'a str,
            language_version: &'a str,
            relay_id: &'a str,
            success_mode: Option<()>,
            test_framework: &'a str,
        }
        self.client
            .client
            .post("https://runner.codewars.com/run")
            .bearer_auth(token)
            .json(&RunPayload {
                ciphered: if fixture_ciphered {
                    &["fixture", "setup"]
                } else {
                    &["setup"]
                },
                code,
                fixture,
                setup: &self.info.package,
                language: &self.info.language_name,
                language_version: &self.info.active_version,
                relay_id: &self.info.solution_id,
                success_mode: None,
                test_framework: &self.info.test_framework,
            })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
    async fn notify(&self, token: &str, code: &str, fixture: &str) -> Result<(), reqwest::Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Payload<'a> {
            code: &'a str,
            fixture: &'a str,
            language_version: &'a str,
            test_framework: &'a str,
            token: &'a str,
        }
        self.client
            .post_with_csrf(format!(
                "https://www.codewars.com/api/v1/code-challenges/projects/{}/solutions/{}/notify",
                &self.project.id, &self.info.solution_id
            ))
            .header(AUTHORIZATION, &self.project.jwt)
            .json(&Payload {
                code,
                fixture,
                language_version: &self.info.active_version,
                test_framework: &self.info.test_framework,
                token,
            })
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
    async fn run(
        &self,
        code: &str,
        fixture_ciphered: bool,
        fixture: &str,
        notify_fixture: &str,
    ) -> Result<TestResult, SessionError> {
        let token = self.auth_runner().await.map_err(|e| SessionError {
            kind: SessionErrKind::AuthRunner,
            inner: e,
        })?;
        let ret = self
            .run_test(&token, code, fixture_ciphered, fixture)
            .await
            .map_err(|e| SessionError {
                kind: SessionErrKind::RunTest,
                inner: e,
            })?;
        self.notify(&ret.token, code, notify_fixture)
            .await
            .map_err(|e| SessionError {
                kind: SessionErrKind::Notify,
                inner: e,
            })?;
        Ok(ret)
    }
    pub async fn test(&self, code: &str, fixture: &str) -> Result<TestResult, SessionError> {
        self.run(code, false, fixture, fixture).await
    }
    pub async fn attempt(&self, code: &str, fixture: &str) -> Result<TestResult, SessionError> {
        self.run(code, true, &self.info.fixture, fixture).await
    }
    /// submit code challenge solution
    pub async fn submit(&self) -> Result<(), reqwest::Error> {
        self.client
            .post_with_csrf(format!(
                "https://www.codewars.com/api/v1/code-challenges/projects/{}/solutions/{}/finalize",
                &self.project.id, &self.info.solution_id
            ))
            .header(AUTHORIZATION, &self.project.jwt)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn project_id() {
        assert_eq!(
            super::find_project_id(include_str!("./project/proj_id_test.js")),
            Some("aaaaaaaaaaaaaaaaaaaaaaaa")
        )
    }
}
