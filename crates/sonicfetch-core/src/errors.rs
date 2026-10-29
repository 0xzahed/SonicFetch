use thiserror::Error;

#[derive(Debug, Error)]
pub enum SonicError {
    #[error("invalid state transition: {0}")]
    InvalidTransition(#[from] LifecycleError),
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("{from:?} -> {to:?} is not permitted")]
pub struct LifecycleError {
    pub from: crate::state::DownloadState,
    pub to: crate::state::DownloadState,
}
