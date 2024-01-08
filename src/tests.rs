use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::path::{Path, PathBuf};

pub const PATH_PREFIX: &str = "./.testing";

// Touch the given test file, creating a dir path to it as we go
pub fn touch_file(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path.parent().unwrap()).ok();
    OpenOptions::new().create(true).write(true).open(path).map(|_| { () })
}

// Convenience function to construct a test path based on PATH_PREFIX
pub fn test_path(suffix: &str) -> PathBuf {
    let mut buf = PathBuf::from(PATH_PREFIX);
    buf.push(suffix);
    buf
}

pub fn create_path(suffix: &str) -> PathBuf {
    let buf = test_path(suffix);
    touch_file(&buf).unwrap();
    buf
}
