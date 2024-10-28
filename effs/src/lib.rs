use pin_project_lite::pin_project;
use std::{
    future::Future,
    path::PathBuf,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

pub mod error;

use error::Error;

pub trait Setup {
    // assumed that an input stream may be more ideal, but for now just take the whole buffer
    fn apply(&self, path: PathBuf) -> Result<Vec<(PathBuf, Filter)>, Error>;
}

pub struct Filter {
    pub(crate) inner: Box<dyn Fn() -> Filtrate>,
}

impl Filter {
    pub fn new(f: impl Fn() -> Filtrate + 'static) -> Self {
        Self { inner: Box::new(f) }
    }

    pub fn get(&self) -> Filtrate {
        (self.inner)()
    }
}

pin_project! {
    pub struct Filtrate {
        #[pin]
        pub(crate) inner: Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>> + Send>>,
    }
}

impl Filtrate {
    pub fn new(fut: impl Future<Output = Result<Vec<u8>, Error>> + Send + 'static) -> Self {
        Self { inner: Box::pin(fut) }
    }
}

impl Future for Filtrate {
    type Output = Result<Vec<u8>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.inner.poll(cx)
    }
}

pub struct Source {
    path: PathBuf,
    setup: Box<dyn Setup>,
}

pub struct Effs {
    mount_point: PathBuf,
    source: Vec<Source>,
}
