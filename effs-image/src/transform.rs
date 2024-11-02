use effs::{
    error::Error,
    EffsSource,
    Entry,
    Filter,
    Filtrate,
    Source,
    Setup,
};
use std::{
    ffi::OsString,
    fs::File,
    io::{
        Read,
        Seek,
        SeekFrom,
    },
    path::PathBuf,
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

impl Setup for Crop {
    fn apply(&mut self, path: PathBuf, request: PathBuf) -> Result<Vec<(OsString, Entry)>, Error> {
        let basename = path.clone()
            .file_name()
            .ok_or(Error::BadSourcePath(path.clone()))?
            .to_owned();
        // TODO actually implement image filter; for now use seek/read length as surrogate placeholder
        let start = self.x as u64;
        let len = self.w;
        Ok(vec![
            (
                basename,
                Filter::new(move || {
                    let path = path.clone();
                    Filtrate::new(
                        async move {
                            let mut file = File::open(&path)?;
                            file.seek(SeekFrom::Start(start))?;
                            let mut output = vec![0; len];
                            file.read(&mut output)?;
                            Ok(output)
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
        let dir = tempdir()?;
        let source = dir.path().join("source");
        let mut source_file = File::create(source.clone())?;
        writeln!(source_file, "0123456789")?;

        let mut effs_source = Source::new(
            source,
            "".into(),
            Crop::new(1, 1, 4, 4),
        );
        let result = effs_source.dir("".into())?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, PathBuf::from("source"));
        let filtrate = match &result[0].1 {
            Entry::Filter(filter) => filter.get().await?,
            _ => unreachable!(),
        };
        assert_eq!(filtrate, b"1234");
        Ok(())
    }
}
