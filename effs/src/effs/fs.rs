use fuse3::{
    raw::prelude::*,
    // Errno,
    Result,
};
use futures_util::stream::{
    self,
    Iter,
};
use std::{
    ffi::{
        OsStr,
        OsString,
    },
    iter::Skip,
    num::NonZeroU32,
    time::Duration,
    vec::IntoIter,
};

use super::Effs;

const TTL: Duration = Duration::from_secs(1);

impl Filesystem for Effs {
    type DirEntryStream<'a> = Iter<IntoIter<Result<DirectoryEntry>>>
    where
        Self: 'a;
    type DirEntryPlusStream<'a> = Iter<Skip<IntoIter<Result<DirectoryEntryPlus>>>>
    where
        Self: 'a;

    async fn init(&self, _req: Request) -> Result<ReplyInit> {
        Ok(ReplyInit {
            max_write: NonZeroU32::new(1024).unwrap(),
        })
    }

    async fn destroy(&self, _req: Request) {}

    async fn lookup(
        &self,
        req: Request,
        parent: u64,
        name: &OsStr,
    ) -> Result<ReplyEntry> {
        tracing::debug!("lookup name={name:?}");

        /*
        // Make it appear that every path lead to some directory.
        let attr = FileAttr {
            ino: 1,
            size: 0,
            blocks: 0,
            atime: SystemTime::now().into(),
            mtime: SystemTime::UNIX_EPOCH.into(),
            ctime: SystemTime::UNIX_EPOCH.into(),
            kind: FileType::Directory,
            perm: fuse3::perm_from_mode_and_kind(FileType::Directory, 0755),
            nlink: 2,
            uid: req.uid,
            gid: req.gid,
            rdev: 0,
            blksize: 0,
        };
        Ok(ReplyEntry {
            ttl: TTL,
            attr: attr,
            generation: 0,
        })
        */
        Err(libc::ENOENT.into())
    }

    async fn getattr(
        &self,
        req: Request,
        inode: u64,
        _fh: Option<u64>,
        _flags: u32,
    ) -> Result<ReplyAttr> {
        // let path = path.ok_or_else(Errno::new_not_exist)?.to_string_lossy();
        tracing::debug!("getattr inode={inode:?}");
        let nodes = self.nodes
            .read()
            .map_err(|_| libc::ENOTRECOVERABLE)?;
        let (_, attr) = nodes.attr_for_inode(inode)?;

        Ok(ReplyAttr {
            ttl: TTL,
            attr,
        })
    }

    async fn readdirplus<'a>(
        &'a self,
        _req: Request,
        parent: u64,
        _fh: u64,
        offset: u64,
        _lock_owner: u64,
    ) -> Result<ReplyDirectoryPlus<Self::DirEntryPlusStream<'a>>> {
        let nodes = self.nodes
            .read()
            .map_err(|_| libc::ENOTRECOVERABLE)?;
        let arena = &nodes.0;

        let cur_nid = nodes.node_id(parent)?;
        let cur_entry = nodes.with_node_id(cur_nid, |(node, attr)| {
            match attr.kind {
                FileType::Directory => Ok(DirectoryEntryPlus {
                    inode: attr.ino,
                    generation: node.generation,
                    kind: attr.kind,
                    name: OsString::from("."),
                    offset: 1,
                    attr: attr,
                    entry_ttl: TTL,
                    attr_ttl: TTL,
                }),
                _ => Err(libc::ENOTDIR.into())
            }
        })?;

        let par_nid = cur_nid.ancestors(arena)
            .next()
            .unwrap_or(cur_nid);
        let par_entry = nodes.with_node_id(par_nid, |(node, attr)| {
            match attr.kind {
                FileType::Directory => Ok(DirectoryEntryPlus {
                    inode: attr.ino,
                    generation: node.generation,
                    kind: attr.kind,
                    name: OsString::from(".."),
                    offset: 2,
                    attr: attr,
                    entry_ttl: TTL,
                    attr_ttl: TTL,
                }),
                _ => Err(libc::ENOTDIR.into())
            }
        })?;

        let entries = vec![Ok(cur_entry), Ok(par_entry)];
        // cur_nid.children(arena);

        Ok(ReplyDirectoryPlus {
            entries: stream::iter(entries.into_iter().skip(offset as usize)),
        })
    }
}
