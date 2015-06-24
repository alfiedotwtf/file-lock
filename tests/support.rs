#![allow(dead_code)]

use std::env;
use std::path::{PathBuf, Path};
use std::os::unix::io::{RawFd, AsRawFd};
use std::fs::{File, OpenOptions, remove_file};


pub struct TempFile {
    inner: File,
    path: PathBuf
}

impl TempFile {
    pub fn new(name: &str) -> TempFile {
        let mut path = env::temp_dir();
        path.push(name);

        TempFile {
            inner: OpenOptions::new()
                               .create(true)
                               .write(true)
                               .open(&path).unwrap(),
            path: path,
        }
    }

    pub fn fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }

    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    pub fn stream(&mut self) -> &mut File {
        &mut self.inner
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        remove_file(&self.path).ok();
    }
}
