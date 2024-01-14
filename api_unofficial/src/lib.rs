//! # Unofficial codewars api
//! Code based on [](https://github.com/kappq/codewars-cli)
use reqwest_cookie_store::CookieStoreRwLock;
use std::sync::Arc;

pub mod project;

mod jwt;

pub mod suggest;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct InitClientError(#[from] reqwest::Error);

pub struct Client {
    client: reqwest::Client,
    cookies: Arc<CookieStoreRwLock>,
}
impl Client {
    pub async fn init(
        log_request: bool,
        session_id: &str,
        remember_user_token: &str,
    ) -> Result<Self, InitClientError> {
        let cookie_store = Arc::new(CookieStoreRwLock::new({
            use reqwest_cookie_store::{CookieStore, RawCookie};
            let mut r = CookieStore::default();
            let url = reqwest::Url::parse("https://www.codewars.com").unwrap();
            r.insert_raw(
                &RawCookie::build("_session_id", session_id)
                    .domain("www.codewars.com")
                    .http_only(true)
                    .finish(),
                &url,
            )
            .unwrap();
            r.insert_raw(
                &RawCookie::build("remember_user_token", remember_user_token)
                    .domain("www.codewars.com")
                    .http_only(true)
                    .finish(),
                &url,
            )
            .unwrap();
            r
        }));
        let client = {
            let ret = reqwest::ClientBuilder::new()
                .cookie_store(true)
                .cookie_provider(Arc::clone(&cookie_store))
                .user_agent(include_str!("./ua.txt"))
                .https_only(true);
            if log_request {
                ret.connection_verbose(true).no_gzip().http1_only()
            } else {
                ret
            }
            .build()?
        };
        Ok(Self {
            client,
            cookies: cookie_store,
        })
    }
    fn post_with_csrf<U: reqwest::IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        let store = self.cookies.read().unwrap();
        let req = self.client.post(url);
        match store.get("www.codewars.com", "/", "CSRF-TOKEN") {
            Some(c) => match percent_encoding::percent_decode_str(c.value()).decode_utf8() {
                Ok(v) => {
                    log::debug!("Added csrf token: {}", v);
                    req.header("X-CSRF-Token", v.as_ref())
                }
                Err(e) => {
                    log::error!(
                        "Failed to decode CSRF-TOKEN cookie: {}, value: {}",
                        e,
                        c.value()
                    );
                    req
                }
            },
            None => {
                log::warn!("csrf token not found for {:?}", req);
                req
            }
        }
    }
}
