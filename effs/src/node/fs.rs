use bytes::Bytes;
use indextree::NodeId;
use fuse3::{
    raw::prelude::*,
    Errno,
    Result,
};
use std::{
    cmp::min,
    ffi::OsStr,
};

use crate::{
    entry::Entry,
    error::NodeLookupError,
};

use super::{
    Node,
    Nodes,
};

impl Nodes {
    pub(crate) fn node_id(&self, inode: u64) -> Result<NodeId> {
        self.basic_node_id(inode)
            .map_err(|_| Errno::from(libc::ENOENT))
    }

    pub(crate) fn lookup_node_id_name(&self, node_id: NodeId, name: &OsStr) -> Result<NodeId> {
        self.basic_lookup_node_id_name(node_id, name)
            .map_err(|e| match e {
                NodeLookupError::NoEntry(..) => Errno::from(libc::ENOENT),
                NodeLookupError::NoSuchName(..) => Errno::from(libc::ENOENT),
                NodeLookupError::NotDirEntry(..) => Errno::from(libc::ENOTDIR),
            })
    }

    pub(crate) fn with_inode<'a, T>(
        &'a self,
        inode: u64,
        handler: impl Fn((&'a Node, FileAttr)) -> Result<T>
    ) -> Result<T> {
        self.with_node_id(
            self.node_id(inode)?,
            handler,
        )
    }

    pub(crate) fn with_node_id<'a, T>(
        &'a self,
        node_id: NodeId,
        handler: impl Fn((&'a Node, FileAttr)) -> Result<T>
    ) -> Result<T> {
        let arena = &self.0;
        let node = &arena[node_id];
        let inner = node.get();
        let kind = match inner.entry
            .as_ref()
            .ok_or_else(|| Errno::from(libc::ENOENT))?
        {
            Entry::Dir(_) => FileType::Directory,
            Entry::Filter(_) => FileType::RegularFile,
            Entry::Filtrated(_) => FileType::RegularFile,
            Entry::PreciseFilter(_) => FileType::RegularFile,
        };
        handler((inner, FileAttr {
            ino: Into::<usize>::into(node_id) as u64,  // FIXME change to usize::from when possible
            size: inner.size
                .unwrap_or(0),
            blocks: 0,
            atime: inner.time,
            mtime: inner.time,
            ctime: inner.time,
            kind: kind,
            perm: fuse3::perm_from_mode_and_kind(kind, inner.mode),
            nlink: 0,
            uid: inner.uid,
            gid: inner.gid,
            rdev: 0,
            blksize: 0,
        }))
    }

    pub(crate) fn attr_for_inode(&self, inode: u64) -> Result<(&Node, FileAttr)> {
        self.attr_for_node_id(
            self.node_id(inode)?
        )
    }

    pub(crate) fn attr_for_node_id(&self, node_id: NodeId) -> Result<(&Node, FileAttr)> {
        self.with_node_id(node_id, Result::Ok)
    }

    pub(crate) async fn read(
        &self,
        node_id: NodeId,
        offset: u64,
        size: u32,
    ) -> Result<Bytes> {
        let arena = &self.0;
        let node = &arena[node_id];
        let inner = node.get();
        match inner.entry
            .as_ref()
            .ok_or_else(|| Errno::from(libc::ENOENT))?
        {
            Entry::Dir(_) => Err(Errno::from(libc::ENOTDIR)),
            Entry::Filter(f) => {
                let r = f.filtrate()
                    .await
                    .map_err(|_| Errno::from(libc::EIO))?;
                Ok(r.slice(offset as usize..min(r.len(), (size as u64 + offset) as usize)))
            }
            Entry::Filtrated(r) => Ok(r.slice(offset as usize..min(r.len(), (size as u64 + offset) as usize))),
            Entry::PreciseFilter(f) => Ok(f.filtrate(offset, size)
                .await
                .map_err(|_| Errno::from(libc::EIO))?),
        }
    }

}
