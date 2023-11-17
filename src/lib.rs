use std::path::{Path, PathBuf};

use anyhow::Result;
use walkdir::WalkDir;

pub fn regular_files(directory: &Path, depth: usize) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    for entry in WalkDir::new(directory)
        .max_depth(depth)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            paths.push(entry.into_path());
        }
    }

    Ok(paths)
}

pub fn directories(directory: &Path, depth: usize) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    for entry in WalkDir::new(directory)
        .max_depth(depth)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() {
            paths.push(entry.into_path());
        }
    }

    Ok(paths)
}
