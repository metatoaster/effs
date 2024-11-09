use fuse3::Timestamp;
use indextree::{
    Arena,
    NodeId,
};
use libc::mode_t;
use std::{
    collections::BTreeMap,
    time::SystemTime,
};

use crate::{
    entry::Entry,
    error::NoSuchNode,
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
