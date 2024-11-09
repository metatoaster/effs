use std::sync::RwLock;

use crate::{
    error::{
        Error,
        SourceError,
    },
    node::Nodes,
    traits::EffsSource,
};

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

mod fs;
