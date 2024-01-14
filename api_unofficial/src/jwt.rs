use crate::Client;

pub(crate) fn find_jwt(input: &str) -> Option<&str> {
    const DICT_KEY: &str = "\\\"jwt\\\"";
    let after_dict_key = &input[input.find(DICT_KEY)? + DICT_KEY.len()..];
    let prefix = after_dict_key
        .trim_start()
        .strip_prefix(':')?
        .trim_start()
        .strip_prefix("\\\"")?;
    Some(prefix.split_once("\\\"")?.0)
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum GetJwtError {
    #[error("failed to send http request")]
    Http(
        #[source]
        #[from]
        reqwest::Error,
    ),
    #[error("Jwt not found")]
    JwtNotFound,
}

impl Client {
    pub(crate) async fn get_jwt<U: reqwest::IntoUrl>(&self, url: U) -> Result<String, GetJwtError> {
        let body = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        log::trace!("jwt body: {}", body);
        Ok(find_jwt(&body).ok_or(GetJwtError::JwtNotFound)?.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn jwt() {
        assert_eq!(
            find_jwt(r#"JSON.parse("{\"username\": \"example\", \"jwt\": \"aaaaa\"}")"#),
            Some("aaaaa")
        );
    }
}
