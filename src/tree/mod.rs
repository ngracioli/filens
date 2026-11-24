use std::{
    ffi::OsStr,
    fs::FileType,
    io,
    path::{Path, PathBuf},
};

pub(crate) struct Entry {
    pub(crate) path: PathBuf,
    pub(crate) kind: FileType,
    pub(crate) depth: usize,
    pub(crate) expanded: bool,
}

impl Entry {
    pub(crate) fn name(&self) -> &OsStr {
        self.path.file_name().unwrap_or(self.path.as_os_str())
    }
}

pub(crate) fn read_directory(path: &Path, depth: usize) -> io::Result<Vec<Entry>> {
    let mut entries = path
        .read_dir()?
        .map(|entry| {
            let entry = entry?;
            Ok(Entry {
                path: entry.path(),
                kind: entry.file_type()?,
                depth,
                expanded: false,
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    entries.sort_unstable_by(|left, right| left.name().cmp(right.name()));
    Ok(entries)
}
