use indextree::NodeId;
use fuse3::{
    raw::prelude::*,
    Errno,
    Result,
};

use crate::entry::Entry;

use super::{
    Node,
    Nodes,
};

impl Nodes {
    pub(crate) fn node_id(&self, inode: u64) -> Result<NodeId> {
        self.basic_node_id(inode)
            .map_err(|_| Errno::from(libc::ENOENT))
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
        };
        handler((inner, FileAttr {
            ino: Into::<usize>::into(node_id) as u64,  // FIXME change to usize::from when possible
            size: 0,
            blocks: 0,
            atime: inner.time,
            mtime: inner.time,
            ctime: inner.time,
            kind: kind,
            perm: fuse3::perm_from_mode_and_kind(kind, 0755),
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
}
