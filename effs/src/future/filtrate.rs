use bytes::Bytes;
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

use crate::error::Error;

pin_project! {
    pub struct Filtrate {
        #[pin]
        pub(crate) inner: Pin<Box<dyn Future<Output = Result<Bytes, Error>> + Send>>,
    }
}

impl Filtrate {
    pub fn new(fut: impl Future<Output = Result<Bytes, Error>> + Send + 'static) -> Self {
        Self { inner: Box::pin(fut) }
    }
}

impl Future for Filtrate {
    type Output = Result<Bytes, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.inner.poll(cx)
    }
}
