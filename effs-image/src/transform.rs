use effs::{
    error::EffectError,
    entry::Entry,
    filter::Filter,
    future::Filtrate,
    source::Source,
    traits::{
        Effect,
        EffsSource,
    },
};
use std::{
    ffi::OsString,
    fs::File,
    io::{
        Read,
        Seek,
        SeekFrom,
    },
    path::{
        Path,
        PathBuf,
    },
};

pub struct Crop {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

impl Crop {
    pub fn new(
        x: usize,
        y: usize,
        w: usize,
        h: usize,
    ) -> Self {
        Self { x, y, w, h }
    }
}

impl Effect for Crop {
    fn apply(&mut self, path: &Path, request: &Path) -> Result<Vec<(OsString, Entry)>, EffectError> {
        let path = path.to_owned();
        let basename = path.clone()
            .file_name()
            .ok_or_else(|| EffectError::BadSourcePath(path.clone(), "no final component found for source"))?
            .to_owned();
        // TODO actually implement image filter; for now use seek/read length as surrogate placeholder
        let start = self.x as u64;
        let len = self.w;
        Ok(vec![
            (
                basename,
                Filter::new(move || {
                    let path = path.to_owned();
                    Filtrate::new(
                        async move {
                            let mut file = File::open(&path)?;
                            file.seek(SeekFrom::Start(start))?;
                            let mut output = vec![0; len];
                            file.read(&mut output)?;
                            Ok(output.into())
                        }
                    )
                }).into()
            )
        ])
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;
    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn crop() -> anyhow::Result<()> {
        let root = tempdir()?;
        let source = root.path().join("source");
        let mut source_file = File::create(source.clone())?;
        writeln!(source_file, "0123456789")?;

        let mut effs_source = Source::new(
            source,
            "".into(),
            Crop::new(1, 1, 4, 4),
        );
        let result = effs_source.dir(Path::new(""))?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, PathBuf::from("source"));
        let filtrate = match &result[0].1 {
            Entry::Filter(filter) => filter.filtrate().await?,
            _ => unreachable!(),
        };
        assert_eq!(filtrate, b"1234".to_vec());
        Ok(())
    }
}
