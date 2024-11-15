use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::{
        File,
        read_dir,
    },
    io::Read as _,
    path::Path,
};

use crate::{
    entry::Entry,
    error::EffectError,
    filter::Filter,
    filtrate::Filtrate,
    traits::Effect,
};

pub struct Mirror;

impl Effect for Mirror {
    fn apply(&mut self, path: &Path, request: &Path) -> Result<Vec<(OsString, Entry)>, EffectError> {
        // XXX assumes the incoming request will not be an absolute path
        if !path.is_dir() {
            return Err(EffectError::BadSourcePath(path.into(), "not a directory"))
        }
        let path = path.join(request);
        if !path.is_dir() {
            return Err(EffectError::BadRequestPath(request.into(), "not a directory"))
        }

        Ok(read_dir(path)?
            .filter_map(|res| {
                res.map(|e| {
                    e.file_type()
                        .map(|file_type| {
                            if file_type.is_dir() {
                                Some((e.file_name(), Entry::Dir(Default::default())))
                            } else if file_type.is_file() {
                                Some((e.file_name(), Entry::Filter(Filter::new(move || {
                                    let path = e.path();
                                    Filtrate::new(
                                        async move {
                                            let mut file = File::open(&path)?;
                                            let mut output = Vec::new();
                                            file.read_to_end(&mut output)?;
                                            Ok(output)
                                        }
                                    )
                                }))))
                            } else {
                                None
                            }
                        })
                        .ok()
                        .flatten()
                })
                    .ok()
                    .flatten()
            })
            .collect::<Vec<_>>()
        )
    }
}
