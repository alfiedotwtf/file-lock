#![allow(dead_code)]
extern crate file_lock;

use std::env;
use std::borrow::Borrow;
use std::path::{PathBuf, Path};
use std::os::unix::io::{RawFd, AsRawFd};
use std::fs::{File, OpenOptions};

use file_lock::Remover;


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

    pub fn stream(&mut self) -> &mut File {
        &mut self.inner
    }
}

impl TempFile<PathBuf> {
    pub fn new(name: &str) -> TempFile<PathBuf> {
        let mut path = env::temp_dir();
        path.push(name);

        TempFile {
            inner: OpenOptions::new()
                               .create(true)
                               .write(true)
                               .open(&path).unwrap(),
            remover: Remover { path: path },
        }
    }
}