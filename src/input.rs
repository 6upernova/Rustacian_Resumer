use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::ports::FileSystem;

pub fn list_dirs(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            paths.push(entry.path());
        }
    }
    Ok(paths)
}

pub fn list_txt_files(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_file() {
            continue;
        }

        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if ext.eq_ignore_ascii_case("txt") {
                paths.push(path);
            }
        }
    }
    Ok(paths)
}

pub fn read_file(path: &Path) -> Result<String, io::Error> {
    fs::read_to_string(path)
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn list_dirs(&self, dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
        list_dirs(dir)
    }

    fn list_txt_files(&self, dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
        list_txt_files(dir)
    }

    fn read_file(&self, path: &Path) -> Result<String, io::Error> {
        read_file(path)
    }
}
