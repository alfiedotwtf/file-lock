use std::path::PathBuf;
use std::fs::File;
use std::io;
use std::fs::OpenOptions;
use std::os::unix::io::{RawFd, AsRawFd};
use std::fmt;
use std::error::Error as ErrorTrait;

use fd;
pub use util::{Mode, Kind, ParseError};

#[derive(Debug)]
pub enum Error {
    LockError(PathBuf, fd::Error),
    IoError(PathBuf, io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::IoError(ref path, ref err) 
                => write!(f, "Couldn't open lock file at '{}': {}", path.display(), err),
            Error::LockError(ref path, ref err)
                => write!(f, "{} (at file '{}')", err, path.display()),
        }
    }
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IoError(_, _) 
                => "Could not open or create the file to lock",
            Error::LockError(_, _)
                => "Failed to obtain a file lock"
        }
    }
}

unsafe impl Send for Error {}

/// A type creating a lock file on demand.
///
/// It supports multiple reader, single writer semantics and encodes 
/// whether read or write access is required in an interface similar 
/// to the one of the [`RwLock`](http://doc.rust-lang.org/std/sync/struct.RwLock.html)
///
/// It will remove the lock file it possibly created in case a lock could be obtained.
#[derive(Debug)]
pub struct Lock {
    path: PathBuf,
    file: Option<File>,
    mode: Mode
}

impl Lock {
    pub fn new(path: PathBuf, mode: Mode) -> Lock {
        Lock {
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
                                   .read(self.mode == Mode::Read)
                                   .write(self.mode == Mode::Write)
                                   .open(&self.path) {
            Err(io_err) => return Err(io_err),
            Ok(file) => (file.as_raw_fd(), Some(file))
        };

        self.file = file;
        Ok(raw_fd)
    }

    pub fn any_lock(&mut self, kind: Kind) -> Result<(), Error> {
        let fd = match self.opened_file_fd() {
            Ok(fd) => fd,
            Err(io_err) => return Err(Error::IoError(self.path.clone(), io_err))
        };

        match fd::lock(fd, kind, self.mode.clone()) {
            Ok(res) => Ok(res),
            Err(lock_err) => Err(Error::LockError(self.path.clone(), lock_err)),
        }
    }

    pub fn lock(&mut self) -> Result<(), Error> {
        self.any_lock(Kind::Blocking)
    }

    pub fn try_lock(&mut self) -> Result<(), Error> {
        self.any_lock(Kind::NonBlocking)
    }

    pub fn unlock(&mut self) -> Result<(), Error> {
        match self.file {
            Some(ref file) => match fd::unlock(file.as_raw_fd()) {
                Ok(res) => Ok(res),
                Err(lock_err) => Err(Error::LockError(self.path.clone(), lock_err)),
            },
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

impl Drop for Lock {
    fn drop(&mut self) {
        self.unlock().ok();
    }
}