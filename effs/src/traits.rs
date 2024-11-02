use std::{
    ffi::OsString,
    path::Path,
};

use crate::{
    entry::Entry,
    error::Error,
};

pub trait Effect {
    /// Take an origin and a request, to produce a list of 2-tuple that maps from a OsString to
    /// an `Entry`, where the entry represents a point to some dir, filter or filtrated bytes.
    ///
    /// `origin` is the source path that points to an existing location on the system (e.g. some
    /// file or directory).
    /// `request` is the subpath relative to the assoicated `Source.dest_path`.  The full path
    /// should lead to some valid Entry::Dir.
    /// that was previously prepared.
    ///
    /// returns a result with a vector containing a listing of pathbufs pointing to their respective
    /// filters.
    fn apply(&mut self, origin: &Path, request: &Path) -> Result<Vec<(OsString, Entry)>, Error>;
}

pub trait EffsSource {
    /// Take an origin and a request, to produce a list of 2-tuple that maps from a OsString to
    /// an `Entry`, where the entry represents a point to some dir, filter or filtrated bytes.
    ///
    /// `request` is the subpath relative to the assoicated `Source.dest_path`.  The full path
    /// should either be the root or lead to some valid Entry::Dir.  If the request hits a path
    /// it should be an error.
    ///
    /// returns a result with a vector containing a listing of pathbufs pointing to their respective
    /// filters.
    fn dir(&mut self, request: &Path) -> Result<Vec<(OsString, Entry)>, Error>;
}
