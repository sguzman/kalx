use thiserror::Error;

#[derive(Debug, Error)]
pub enum KalxError {
    #[error("authentication is required for this command")]
    MissingAuth,
}
