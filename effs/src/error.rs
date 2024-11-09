use std::path::PathBuf;
use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Internal Error")]
    Internal,
    #[error(transparent)]
    Source(#[from] SourceError),
    #[error(transparent)]
    NoSuchNode(#[from] NoSuchNode),
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SourceError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Bad Request Path: {0}; Reason: {1}")]
    BadRequestPath(PathBuf, &'static str),
    #[error(transparent)]
    Effect(#[from] EffectError),
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum EffectError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Bad Source Path: {0}; Reason: {1}")]
    BadSourcePath(PathBuf, &'static str),
    #[error("Bad Request Path: {0}; Reason: {1}")]
    BadRequestPath(PathBuf, &'static str),
}

#[derive(Debug, Error)]
#[error("No Such Node: {0}")]
pub struct NoSuchNode(pub(crate) u64);
