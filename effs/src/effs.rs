use fuse3::{
    raw::prelude::*,
    Errno,
    Result,
    Timestamp,
};
use futures_util::stream::{
    self,
    Iter,
};
use indextree::{
    self,
    Arena,
    NodeId,
};
use libc::mode_t;
use std::{
    collections::BTreeMap,
    ffi::{
        OsStr,
        OsString,
    },
    iter::Skip,
    num::NonZeroU32,
    path::PathBuf,
    sync::RwLock,
    time::{
        Duration,
        SystemTime,
    },
    vec::IntoIter,
};

use crate::{
    entry::Entry,
    traits::EffsSource,
};

const TTL: Duration = Duration::from_secs(1);

pub struct Node {
    entry: Option<Entry>,
    // this should be incremented by the arena for the replacement node at the same NodeId
    generation: u64,

    size: u64,
    time: Timestamp,
    uid: u32,
    gid: u32,
    mode: mode_t,
    // perm: fuse3::perm_from_mode_and_kind(FileType::Directory, 0755),
}

impl Node {
    pub fn link(&mut self, entry: Entry) {
        self.mode = match entry {
            Entry::Dir(_) => 0755,
            Entry::Filter(_) => 0644,
            Entry::Filtrated(_) => 0644,
        };
        self.time = SystemTime::now().into();
        self.generation += 1;
        self.entry = Some(entry);
    }
}

impl Default for Node {
    fn default() -> Self {
        Self {
            entry: None,
            generation: 0,

            size: 0,
            time: Timestamp::new(0, 0),
            gid: 0,
            uid: 0,
            mode: 0,
        }
    }
}

pub struct Effs {
    source: RwLock<Vec<Box<dyn EffsSource>>>,
    nodes: RwLock<EffsNodes>,
}

pub struct EffsNodes(Arena<Node>);

impl EffsNodes {
    fn node_id(&self, inode: u64) -> Result<NodeId> {
        let arena = &self.0;
        let index = inode as usize;
        arena
            .get_node_id_at(
                index.try_into()
                    .map_err(|_| Errno::from(libc::ENOENT))?
            )
            .ok_or_else(|| Errno::from(libc::ENOENT))
    }

    fn with_inode<'a, T>(
        &'a self,
        inode: u64,
        handler: impl Fn((&'a Node, FileAttr)) -> Result<T>
    ) -> Result<T> {
        self.with_node_id(
            self.node_id(inode)?,
            handler,
        )
    }

    fn with_node_id<'a, T>(
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

    fn attr_for_inode(&self, inode: u64) -> Result<(&Node, FileAttr)> {
        self.attr_for_node_id(self.node_id(inode)?)
    }

    fn attr_for_node_id(&self, node_id: NodeId) -> Result<(&Node, FileAttr)> {
        self.with_node_id(node_id, Result::Ok)
    }
}

impl Default for Effs {
    fn default() -> Self {
        let mut arena = Arena::new();
        let root = arena.new_node(Node::default());
        let node = &mut arena[root];
        node.get_mut().link(Entry::Dir(BTreeMap::new()));

        // `FUSE_ROOT_ID` is defined as 1
        let node_id: usize = root.into();
        assert!(node_id == 1);

        Self {
            source: RwLock::new(Vec::new()),
            nodes: RwLock::new(EffsNodes(arena)),
        }
    }
}

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
