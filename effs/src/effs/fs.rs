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
            .await;
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
            .await;
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
        // TODO this block should only run when the configuration to always
        // refresh when readdirplus is called be enabled.
        // Currently, this enables the dynamic generation of new listings, but
        // not exactly in a friendly manner.
        {
            let path = self.path_of_inode(parent)
                // this error implies the parent inode disappeared
                .await
                .map_err(|_| libc::ENOENT)?;
            // TODO log this error?
            self.build_nodes(&path)
                .await
                .map_err(|_| libc::ENOTRECOVERABLE)?;
        }
        let nodes = self.nodes
            .read()
            .await;
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

    async fn open(&self, _req: Request, inode: u64, flags: u32) -> Result<ReplyOpen> {
        let nodes = self.nodes
            .read()
            .await;
        let _ = nodes.node_id(inode)?;

        // enable FOPEN_DIRECT_IO
        let flags = flags | 1;

        // TODO set up the future for read to use?
        Ok(ReplyOpen { fh: 0, flags })
    }

    async fn read(
        &self,
        _req: Request,
        inode: u64,
        // TODO use fh to deal with caching the future that may have been created?
        _fh: u64,
        offset: u64,
        size: u32,
    ) -> Result<ReplyData> {
        let nodes = self.nodes
            .read()
            .await;
        let node_id = nodes.node_id(inode)?;
        let data = nodes.read(node_id, offset, size).await?;
        tracing::debug!("read inode={inode} offset={offset} size={size} got data.len()={}", data.len());
        Ok(ReplyData { data: data.into() })
    }

}
