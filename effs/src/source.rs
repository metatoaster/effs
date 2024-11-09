use std::{
    ffi::OsString,
    path::{
        Path,
        PathBuf,
    },
};

use crate::{
    entry::Entry,
    error::SourceError,
    traits::{
        Effect,
        EffsSource,
    },
};

pub struct Source<S> {
    // This is the source file
    source_path: PathBuf,
    // This is the destination path, with the root at (or relative to) the mount point.
    dest_path: PathBuf,
    // Additional struct providing the data required for the filter setup.
    setup: S,
    // TODO cache goes here?
}

impl<S> Source<S> {
    pub fn new(
        source_path: PathBuf,
        dest_path: PathBuf,
        setup: S,
    ) -> Self {
        Self {
            source_path,
            dest_path,
            setup,
        }
    }
}

impl<E> EffsSource for Source<E>
where
    E: Effect
{
    fn dir(&mut self, request: &Path) -> Result<Vec<(OsString, Entry)>, SourceError> {
        Ok(self.setup.apply(self.source_path.as_path(), request)?)
    }
}
