#![allow(dead_code)]
extern crate file_lock;

use std::env;
use std::borrow::Borrow;
use std::path::{PathBuf, Path};
use std::os::unix::io::{RawFd, AsRawFd};
use std::fs::{File, OpenOptions, remove_file};

use file_lock::AccessMode;

/// A utility type to assure the removal of a file.
///
/// It is useful when a temporary lock file is created. When an instance dropped
/// of this type is dropped, the lock file will be removed. It is not an error
/// if the file doesn't exist anymore.
///
/// TODO(ST): remove Remover - it's better as part of TempFile
pub struct Remover<P: Borrow<PathBuf>> {
  pub path: P,
}

impl<P> Drop for Remover<P> 
    where P: Borrow<PathBuf> {
    fn drop(&mut self) {
        remove_file(self.path.borrow()).ok();
    }
}


pub struct TempFile<P: Borrow<PathBuf>> {
    inner: File,
    remover: Remover<P>
}

impl<P> TempFile<P> where P: Borrow<PathBuf> {

    pub fn fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }

    pub fn path(&self) -> &Path {
        self.remover.path.borrow()
    }

    pub fn path_buf(&self) -> PathBuf {
        self.remover.path.borrow().clone()
    }

    pub fn stream(&mut self) -> &mut File {
        &mut self.inner
    }
}

impl TempFile<PathBuf> {
    pub fn new(name: &str, mode: AccessMode) -> TempFile<PathBuf> {
        let mut path = env::temp_dir();
        path.push(name);

        TempFile {
            inner: OpenOptions::new()
                               .create(true)
                               .read(mode == AccessMode::Read)
                               .write(mode == AccessMode::Write)
                               .open(&path).unwrap(),
            remover: Remover { path: path },
        }
    }
}