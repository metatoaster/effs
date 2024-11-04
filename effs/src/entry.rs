use std::{
    collections::BTreeMap,
    ffi::OsString,
    sync::Arc,
};

use crate::filter::Filter;

#[derive(Clone)]
pub enum Entry {
    Dir(BTreeMap<OsString, u64>),
    // TODO some kind of wrapper around this for size hints?
    // certain kinds of filtering will provide size-hints before the whole thing is
    // read into memory (e.g. archive files), but for image manipulation this would not
    // be available until the filter is called, then the size is set.  It may be useful
    // to store the size somehow.  Or be lazy and provide the size equal to the original
    // source file.
    Filter(Filter),
    Filtrated(Arc<[u8]>),
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
