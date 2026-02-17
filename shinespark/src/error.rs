use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("internal error: {0:#?}")]
    Internal(#[from] anyhow::Error),

    #[error("configuration error: {0:#?}")]
    Config(anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
