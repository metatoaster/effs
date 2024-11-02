use std::sync::Arc;

use crate::filtrate::Filtrate;

#[derive(Clone)]
pub struct Filter {
    pub(crate) inner: Arc<dyn Fn() -> Filtrate>,
}

impl Filter {
    pub fn new(f: impl Fn() -> Filtrate + 'static) -> Self {
        Self { inner: Arc::new(f) }
    }

    pub fn get(&self) -> Filtrate {
        (self.inner)()
    }
}
