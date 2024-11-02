use std::path::PathBuf;

use crate::traits::EffsSource;

pub struct Effs {
    mount_point: PathBuf,
    source: Vec<Box<dyn EffsSource>>,
}
