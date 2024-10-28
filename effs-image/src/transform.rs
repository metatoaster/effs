use effs::{
    error::Error,
    Filter,
    Filtrate,
    Setup,
};
use std::{
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
    fn apply(&self, path: PathBuf) -> Result<Vec<(PathBuf, Filter)>, Error> {
        let basename: PathBuf = path.clone()
            .file_name()
            .ok_or(Error::BadSourcePath(path.clone()))?
            .to_owned()
            .into();
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
                })
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

        let filter = Crop::new(1, 1, 4, 4);
        let result = filter.apply(source)?;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, PathBuf::from("source"));
        let filtrate = result[0].1.get().await?;
        assert_eq!(filtrate, b"1234");
        Ok(())
    }
}
