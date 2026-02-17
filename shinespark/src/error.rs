use std::borrow::Cow;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("internal error: {0:#?}")]
    Internal(#[from] anyhow::Error),

    #[error("configuration error: {0:#?}")]
    Config(anyhow::Error),

    #[error("crypto error: {0:#?}")]
    Crypto(anyhow::Error),

    #[error("unauthorized")]
    UnAuthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("invalid parameter: {0}")]
    InvalidParameter(Cow<'static, str>),

    #[error("illegal state: {0}")]
    IllegalState(Cow<'static, str>),

    #[error("already exists: {0}")]
    AlreadyExists(Cow<'static, str>),

    #[error("not found: {0}")]
    NotFound(Cow<'static, str>),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {

    #[test]
    fn test_anyhow_context() {
        let error = anyhow::anyhow!("test");
        let error_with_context = error.context("context").context("one more");
        let internal = crate::Error::Internal(error_with_context);
        println!("{}", internal);
    }
}
