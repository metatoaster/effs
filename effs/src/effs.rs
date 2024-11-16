use indextree::NodeId;
use std::{
    path::{
        Component,
        Path,
        PathBuf,
    },
};
use tokio::sync::RwLock;

use crate::{
    error::Error,
    node::{
        Node,
        Nodes,
    },
    traits::EffsSource,
};

mod fs;

pub struct Effs {
    sources: RwLock<Vec<Box<dyn EffsSource>>>,
    nodes: RwLock<Nodes>,
}

impl Default for Effs {
    fn default() -> Self {
        Self {
            sources: RwLock::new(Vec::new()),
            nodes: RwLock::new(Nodes::default()),
        }
    }
}

impl Effs {
    pub async fn push_source(&self, source: impl EffsSource) -> Result<(), Error> {
        let mut sources = self.sources
            .write()
            .await;
        sources.push(Box::new(source));
        Ok(())
    }

    pub(crate) async fn path_of_inode(&self, inode: u64) -> Result<PathBuf, Error> {
        let nodes = self.nodes
            .read()
            .await;
        Ok(nodes.path_of_inode(inode)?)
    }

    async fn path_to_node_id(&self, path: &Path) -> Result<NodeId, Error> {
        let nodes = self.nodes
            .read()
            .await;

        let mut comps = path.components().peekable();
        if comps.peek() == Some(&Component::RootDir) {
            // discard the root component
            comps.next();
        }
        let mut result = nodes.basic_node_id(1)?;
        // XXX this assumes the incoming path is fully normalize, i.e. without
        // Component::ParentDir in the mix
        while let Some(Component::Normal(fragment)) = comps.next() {
            result = nodes.basic_lookup_node_id_name(result, fragment)?;
        }

        Ok(result)
    }

    pub async fn build_nodes(&self, path: &Path) -> Result<(), Error> {
        let path = if path.starts_with("/") {
            path.strip_prefix("/")
                .expect("base somehow wasn't start_with \"/\"")
        } else {
            path
        };
        let par_node_id = self.path_to_node_id(path).await?;

        let mut sources = self.sources
            .write()
            .await;
        let mut nodes = self.nodes
            .write()
            .await;
        let process = sources.iter_mut()
            .filter_map(|source| {
                // TODO figure out how to deal with error here
                // TODO should probably log the error
                source.dir(path)
                    .map(|r| r.into_iter())
                    .ok()
            })
            .flatten();
        for (name, entry) in process {
            // TODO should probably log the error
            nodes.link_entry(par_node_id, name, entry).ok();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn basic() -> anyhow::Result<()> {
        let fs = Effs::default();
        assert_eq!(1usize, fs.path_to_node_id(Path::new("")).await?.into());
        assert_eq!(1usize, fs.path_to_node_id(Path::new("/")).await?.into());
        assert!(fs.path_to_node_id(Path::new("no_such_path")).await.is_err());
        assert!(fs.path_to_node_id(Path::new("/no_such_path")).await.is_err());
        Ok(())
    }
}
