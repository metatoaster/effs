use std::sync::Arc;

use crate::filtrate::Filtrate;

/// The standard filter, one where the full output will be produced
#[derive(Clone)]
pub struct Filter {
    pub(crate) inner: Arc<dyn Fn() -> Filtrate + Send + Sync>,
}

impl Filter {
    pub fn new(f: impl Fn() -> Filtrate + Send + Sync + 'static) -> Self {
        Self { inner: Arc::new(f) }
    }

    pub fn get(&self) -> Filtrate {
        (self.inner)()
    }
}

/// A version of filter that allows the offset and size be passed and is smart enough
/// to handle them to return the specific requested slice.
#[derive(Clone)]
pub struct PreciseFilter {
    pub(crate) inner: Arc<dyn Fn(u64, u32) -> Filtrate + Send + Sync>,
}

impl PreciseFilter {
    pub fn new(f: impl Fn(u64, u32) -> Filtrate + Send + Sync + 'static) -> Self {
        Self { inner: Arc::new(f) }
    }

    pub fn get(&self, offset: u64, size: u32) -> Filtrate {
        (self.inner)(offset, size)
    }
}
