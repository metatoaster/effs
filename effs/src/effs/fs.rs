use fuse3::{
    raw::prelude::*,
    Errno,
    Result,
};
use futures_util::{
    StreamExt,
    stream::{
        self,
        Iter,
    },
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

use crate::entry::Entry;
use super::Effs;

const TTL: Duration = Duration::from_secs(1);

impl Filesystem for Effs {
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

    async fn lookup(
        &self,
        _req: Request,
        parent: u64,
        name: &OsStr,
    ) -> Result<ReplyEntry> {
        tracing::debug!("lookup parent={parent} name={name:?}");
        let nodes = self.nodes
            .read()
            .map_err(|_| libc::ENOTRECOVERABLE)?;
        let (node, attr) = nodes.attr_for_node_id(
            nodes.lookup_node_id_name(
                nodes.node_id(parent)?,
                name,
            )?
        )?;
        Ok(ReplyEntry {
            ttl: TTL,
            attr: attr,
            generation: node.generation,
        })
    }

    async fn getattr(
        &self,
        _req: Request,
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

        let pre_entries = vec![Ok(cur_entry), Ok(par_entry)].into_iter();

        // TODO use stream::iter to do this in one slurp rather than buffering this
        let entries = pre_entries
            .chain(
                cur_nid.children(arena)
                    .filter_map(|nid| nodes.attr_for_node_id(nid).ok())
                    .enumerate()
                    .map(|(i, (node, attr))| {
                        Ok(DirectoryEntryPlus {
                            inode: attr.ino,
                            generation: node.generation,
                            kind: attr.kind,
                            name: node.name.clone(),
                            offset: i as i64 + 3,
                            attr: attr,
                            entry_ttl: TTL,
                            attr_ttl: TTL,
                        })
                    })
            )
            .skip(offset as usize)
            .collect::<Vec<_>>();

        Ok(ReplyDirectoryPlus {
            entries: stream::iter(entries),
        })
    }
}
