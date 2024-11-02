use fuse3::{
    path::prelude::*,
    Result,
};
use futures_util::stream::Iter;
use std::{
    num::NonZeroU32,
    path::PathBuf,
    vec::IntoIter,
};

use crate::traits::EffsSource;

#[derive(Default)]
pub struct Effs {
    mount_point: PathBuf,
    source: Vec<Box<dyn EffsSource>>,
}

impl PathFilesystem for Effs {
    type DirEntryStream<'a> = Iter<IntoIter<Result<DirectoryEntry>>>
    where
        Self: 'a;
    type DirEntryPlusStream<'a> = Iter<IntoIter<Result<DirectoryEntryPlus>>>
    where
        Self: 'a;

    async fn init(&self, _req: Request) -> Result<ReplyInit> {
        Ok(ReplyInit {
            max_write: NonZeroU32::new(1024).unwrap(),
        })
    }

    async fn destroy(&self, _req: Request) {}
}
