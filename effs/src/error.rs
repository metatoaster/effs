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
    #[error(transparent)]
    NodeLookupError(#[from] NodeLookupError),
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

#[derive(Debug, Error)]
pub enum NodeLookupError {
    // the id is a valid node_id but no entry is associated with it
    #[error("No Entry: NodeId={0}")]
    NoEntry(u64),
    // name is not found at the node_id
    #[error("No Such Name at Node: NodeId={0}, name={1}")]
    NoSuchName(u64, String),
    // name does not lead to a valid Entry::Dir at the node_id
    #[error("Not Dir Entry: NodeId={0}")]
    NotDirEntry(u64),
}
