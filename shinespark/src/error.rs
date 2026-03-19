use std::borrow::Cow;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("internal error: {0:#?}")]
    Internal(anyhow::Error),

    #[error("illegal state: {0}")]
    IllegalState(Cow<'static, str>),

    #[error("not implemented")]
    NotImplemented,
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use anyhow::Context;

    #[test]
    fn test_anyhow_context() {
        let error = anyhow::anyhow!("test");
        let error = error.context("context");
        let error = crate::error::Error::Internal(error);
        print!("{:?}", error);
    }

    #[test]
    fn test_error_with_context() {
        let error = crate::error::Error::IllegalState(std::borrow::Cow::Borrowed("some"));
        let error = anyhow::anyhow!(error).context("context");
        println!("{:?}", error);
    }

    fn do_some_work() -> crate::error::Result<()> {
        Err(crate::error::Error::IllegalState(
            std::borrow::Cow::Borrowed("some"),
        ))
    }

    #[test]
    fn test_wrap_context() {
        let err = do_some_work().context("wrap");
        println!("{:?}", err);
    }
}
