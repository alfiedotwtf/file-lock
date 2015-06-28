use std::path::PathBuf;
use std::fs::File;
use std::io;
use std::fs::OpenOptions;
use std::os::unix::io::{RawFd, AsRawFd};
use std::fmt;
use std::error::Error as ErrorTrait;

use lock::{self, LockKind, AccessMode, lock, unlock};

#[derive(Debug)]
pub enum Error {
    LockError(lock::Error),
    IoError(PathBuf, io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::IoError(ref path, ref err) 
                => write!(f, "Couldn't open lock file at '{}': {}", path.display(), err),
            Error::LockError(ref err)
                => err.fmt(f),
        }
    }
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IoError(_, _) 
                => "Could not open or create the file to lock",
            Error::LockError(_)
                => "Failed to obtain a file lock"
        }
    }
}

unsafe impl Send for Error {}

impl From<lock::Error> for Error {
    fn from(err: lock::Error) -> Self {
        Error::LockError(err)
    }
}

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
    mode: AccessMode
}

impl FileLock {
    pub fn new(path: PathBuf, mode: AccessMode) -> FileLock {
        FileLock {
            path: path,
            file: None,
            mode: mode,
        }
    }

    fn opened_file_fd(&mut self) -> Result<RawFd, io::Error> {
        if let Some(ref file) = self.file {
            return Ok(file.as_raw_fd())
        }

        let (raw_fd, file) = match OpenOptions::new()
                                   .create(true)
                                   .read(self.mode == AccessMode::Read)
                                   .write(self.mode == AccessMode::Write)
                                   .open(&self.path) {
            Err(io_err) => return Err(io_err),
            Ok(file) => (file.as_raw_fd(), Some(file))
        };

        self.file = file;
        Ok(raw_fd)
    }

    pub fn any_lock(&mut self, kind: LockKind) -> Result<(), Error> {
        let fd = match self.opened_file_fd() {
            Ok(fd) => fd,
            Err(io_err) => return Err(Error::IoError(self.path.clone(), io_err))
        };

        Ok(try!(lock(fd, kind, self.mode.clone())))
    }

    pub fn lock(&mut self) -> Result<(), Error> {
        self.any_lock(LockKind::Blocking)
    }

    pub fn try_lock(&mut self) -> Result<(), Error> {
        self.any_lock(LockKind::NonBlocking)
    }

    pub fn unlock(&mut self) -> Result<(), Error> {
        match self.file {
            Some(ref file) => Ok(try!(unlock(file.as_raw_fd()))),
            None => Err(Error::IoError(self.path.clone(),
                                       io::Error::new(io::ErrorKind::NotFound, 
                                       "unlock() called before lock() or try_lock()").into()))
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn file(&mut self) -> Option<&mut File> {
        self.file.as_mut()
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        self.unlock().ok();
    }
}