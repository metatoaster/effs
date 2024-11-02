use pin_project_lite::pin_project;
use std::{
    ffi::OsString,
    future::Future,
    path::{
        Path,
        PathBuf,
    },
    pin::Pin,
    sync::Arc,
    task::{
        Context,
        Poll,
    },
};

pub mod error;

use error::Error;

pub trait Setup {
    /// Take an origin and a request, to produce a list of 2-tuple that maps from a PathBuf to
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
    fn apply(&mut self, origin: PathBuf, request: PathBuf) -> Result<Vec<(OsString, Entry)>, Error>;
}

#[derive(Clone)]
pub struct Filter {
    pub(crate) inner: Arc<dyn Fn() -> Filtrate>,
}

pin_project! {
    pub struct Filtrate {
        #[pin]
        pub(crate) inner: Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>> + Send>>,
    }
}

#[derive(Clone)]
pub enum Entry {
    Dir,
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

impl Filter {
    pub fn new(f: impl Fn() -> Filtrate + 'static) -> Self {
        Self { inner: Arc::new(f) }
    }

    pub fn get(&self) -> Filtrate {
        (self.inner)()
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

pub trait EffsSource {
    /// Take an origin and a request, to produce a list of 2-tuple that maps from a PathBuf to
    /// an `Entry`, where the entry represents a point to some dir, filter or filtrated bytes.
    ///
    /// `request` is the subpath relative to the assoicated `Source.dest_path`.  The full path
    /// should either be the root or lead to some valid Entry::Dir.  If the request hits a path
    /// it should be an error.
    ///
    /// returns a result with a vector containing a listing of pathbufs pointing to their respective
    /// filters.
    fn dir(&mut self, request: PathBuf) -> Result<Vec<(OsString, Entry)>, Error>;
}

impl<S> EffsSource for Source<S>
where
    S: Setup
{
    fn dir(&mut self, request: PathBuf) -> Result<Vec<(OsString, Entry)>, Error> {
        self.setup.apply(self.source_path.clone(), request.clone())
    }
}

pub struct Effs {
    mount_point: PathBuf,
    source: Vec<Box<dyn EffsSource>>,
}
