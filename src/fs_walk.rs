use std::fs;
use std::path::{Path, PathBuf};

pub fn collect_files(root: &Path, recursive: bool) -> Vec<PathBuf> {
    if recursive && root.is_dir() {
        let mut out = Vec::new();
        collect_recursive(root, &mut out);
        out
    } else if root.is_file() {
        vec![root.to_path_buf()]
    } else {
        Vec::new()
    }
}

fn collect_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_recursive(&path, out);
        } else if path.is_file() {
            out.push(path);
        }
    }
}
