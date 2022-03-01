use thiserror::Error;

#[derive(Debug, Error)]
pub enum PrerenderError {
    #[error("Invalid Url.")]
    InvalidUrl,

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}
