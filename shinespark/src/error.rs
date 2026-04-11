use std::borrow::Cow;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("illegal state: {0}")]
    IllegalState(Cow<'static, str>),

    #[error("not implemented")]
    NotImplemented,

    #[error("un authorized")]
    UnAuthorized,

    #[error("database error: {0}")]
    DatabaseError(anyhow::Error),

    #[error("not found")]
    NotFound,

    #[error("already exists")]
    AlreadyExists,

    #[error("invalid credentials")]
    InvalidCredentials,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn code(&self) -> &'static str {
        match self {
            Error::Internal(_) => "INTERNAL",
            Error::IllegalState(_) => "ILLEGAL_STATE",
            Error::NotImplemented => "NOT_IMPLEMENTED",
            Error::UnAuthorized => "UNAUTHORIZED",
            Error::DatabaseError(_) => "DATABASE_ERROR",
            Error::NotFound => "NOT_FOUND",
            Error::AlreadyExists => "ALREADY_EXISTS",
            Error::InvalidCredentials => "INVALID_CREDENTIALS",
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    #[test]
    fn test_anyhow_context() {
        let error = anyhow::anyhow!("test");
        let error = error.context("context");
        let _error = crate::error::Error::Internal(error);
        print!("{:?}", _error);
    }

    #[test]
    fn test_error_with_context() {
        let error = crate::error::Error::IllegalState(std::borrow::Cow::Borrowed("some"));
        let _error = anyhow::anyhow!(error).context("context");
        //println!("{:?}", error);
    }

    fn do_some_work() -> crate::error::Result<()> {
        Err(crate::error::Error::IllegalState(
            std::borrow::Cow::Borrowed("some"),
        ))
    }

    #[test]
    fn test_wrap_context() {
        let _err = do_some_work().context("wrap");
        //println!("{:?}", err);
    }
}
