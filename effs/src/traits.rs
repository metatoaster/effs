use std::{
    ffi::OsString,
    path::Path,
};

use crate::{
    entry::Entry,
    error::{
        EffectError,
        SourceError,
    },
};

/// Apply an effect to a source file or directory.
///
/// Given an origin and the request subpath, produce a list of 2-tuple that maps from a OsString
/// to an `Entry`, where the entry represents a point to some dir, filter or filtrated bytes.
///
/// `origin` is the source path that points to an existing location on the system (e.g. some
/// file or directory).
/// `request` is the subpath relative to the assoicated `Source.dest_path`.  The full path
/// should lead to some valid Entry::Dir that was returned earlier, and if it's something else
/// an error may happen.
///
/// Returns a result with a vector containing a listing of pathbufs pointing to their respective
/// filters.
pub trait Effect<Error=EffectError>: Send + Sync + 'static {
    fn apply(&mut self, origin: &Path, request: &Path) -> Result<Vec<(OsString, Entry)>, Error>;
}

/// Serves as file sources for Effs.
///
/// Given a `request` path, produce a produce a list of 2-tuple that maps from a OsString
/// to an `Entry`, where the entry represents a point to some dir, filter or filtrated bytes.
///
/// `request` is the subpath relative to the assoicated `Source.dest_path`.  The full path
/// should either be the root or lead to some valid Entry::Dir.  If the request hits a filter
/// or filtrated an error will happen.
///
/// returns a result with a vector containing a listing of pathbufs pointing to their respective
/// filters.
pub trait EffsSource<Error=SourceError>: Send + Sync + 'static {
    fn dir(&mut self, request: &Path) -> Result<Vec<(OsString, Entry)>, Error>;
}
