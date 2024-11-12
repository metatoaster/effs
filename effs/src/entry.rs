use std::{
    collections::BTreeMap,
    ffi::OsString,
    sync::Arc,
};

use crate::filter::{
    Filter,
    PreciseFilter,
};

#[derive(Clone)]
pub enum Entry {
    /// A directory listing.  It maps from some name to an inode.
    Dir(BTreeMap<OsString, u64>),
    /// A standard naive filter, it provides a function that produces a
    /// future that will retrieve the entirety of some output on demand.
    Filter(Filter),
    /// A completely cached output through a filter.
    Filtrated(Arc<[u8]>),
    /// A version of filter that can be precise about what to retrieve;
    /// rather than providing a future that will retrieve the entirety
    /// of the output, additional offset and size argument must be
    /// provided to retrieve the desired output.
    PreciseFilter(PreciseFilter),
}

impl From<Filter> for Entry {
    fn from(f: Filter) -> Self {
        Self::Filter(f)
    }
}

impl From<Arc<[u8]>> for Entry {
    fn from(f: Arc<[u8]>) -> Self {
        Self::Filtrated(f)
    }
}
