use std::path::PathBuf;
use std::fs::{File, remove_file};

/// A type creating a lock file on demand.
///
/// It supports multiple reader, single writer semantics and encodes 
/// whether read or write access is required in an interface similar 
/// to the one of the [`RwLock`](http://doc.rust-lang.org/std/sync/struct.RwLock.html)
///
/// It will remove the lock file it possibly created in case a lock could be obtained.
#[derive(Debug)]
pub struct FileLock {
    path: PathBuf,
    file: Option<File>,
}

impl FileLock {
    fn new(path: PathBuf) -> FileLock {
        FileLock {
            path: path,
            file: None,
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        remove_file(&self.path).ok();
    }
}