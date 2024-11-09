use fuse3::Timestamp;
use indextree::{
    Arena,
    NodeId,
};
use libc::mode_t;
use std::{
    ffi::OsStr,
    collections::BTreeMap,
    time::SystemTime,
};

use crate::{
    entry::Entry,
    error::{
        NoSuchNode,
        NodeLookupError,
    },
};

mod fs;

pub struct Node {
    pub(crate) entry: Option<Entry>,
    // this should be incremented by the arena for the replacement node at the same NodeId
    pub(crate) generation: u64,

    pub(crate) size: u64,
    pub(crate) time: Timestamp,
    pub(crate) uid: u32,
    pub(crate) gid: u32,
    pub(crate) mode: mode_t,
    // perm: fuse3::perm_from_mode_and_kind(FileType::Directory, 0755),
}

pub struct Nodes(pub(crate) Arena<Node>);

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

impl Nodes {
    pub(crate) fn basic_node_id(&self, inode: u64) -> Result<NodeId, NoSuchNode> {
        let arena = &self.0;
        let index = inode as usize;
        arena
            .get_node_id_at(
                index.try_into().map_err(|_| NoSuchNode(inode))?
            )
            .ok_or_else(|| NoSuchNode(inode))
    }

    pub(crate) fn basic_lookup_node_id_name(
        &self,
        node_id: NodeId,
        name: &OsStr,
    ) -> Result<NodeId, NodeLookupError> {
        let arena = &self.0;
        let node = &arena[node_id];
        let inner = node.get();
        let dir = match inner.entry
            .as_ref()
            .ok_or_else(|| NodeLookupError::NoEntry(
                Into::<usize>::into(node_id) as u64
            ))?
        {
            Entry::Dir(dir) => Ok(dir),
            _ => Err(NodeLookupError::NotDirEntry(
                Into::<usize>::into(node_id) as u64
            )),
        }?;

        self.basic_node_id(
            *dir.get(name)
                // Simple lookup error.
                .ok_or_else(|| NodeLookupError::NoSuchName(
                    Into::<usize>::into(node_id) as u64,
                    name.to_string_lossy().to_string(),
                ))?
        )
            // While the name exists and points to some node_id, it does
            // not in fact exists in the arena.
            .map_err(|_| NodeLookupError::NoSuchName(
                Into::<usize>::into(node_id) as u64,
                name.to_string_lossy().to_string(),
            ))
    }
}

impl Default for Nodes {
    fn default() -> Self {
        let mut arena = Arena::new();
        let root = arena.new_node(Node::default());
        let node = &mut arena[root];
        node.get_mut().link(Entry::Dir(BTreeMap::new()));

        // `FUSE_ROOT_ID` is defined as 1
        let node_id: usize = root.into();
        assert!(node_id == 1);
        Self(arena)
    }
}
