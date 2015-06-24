extern crate file_lock;
extern crate libc;

use file_lock::*;

use std::env;
use std::fs::{File, OpenOptions, remove_file};
use std::path::PathBuf;
use std::os::unix::io::{RawFd, AsRawFd};



struct TempFile {
    inner: File,
    path: PathBuf
}

impl TempFile {
    fn new(name: &str) -> TempFile {
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

    fn fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        remove_file(&self.path).ok();
    }
}


//
// unfortunately we can't abstract this out for lock() and lock_wait()
// into a macro because string concat doesn't exist
//

// lock_wait() tests

#[test]
fn invalid_fd() {
    for fd in &[-1 as RawFd, 40125] {
        for kind in &[LockKind::Blocking, LockKind::NonBlocking] {
            assert_eq!(Lock::new(*fd).lock(kind.clone(), AccessMode::Write), 
                       Err(Error::Errno(libc::consts::os::posix88::EBADF)));
        }

        assert_eq!(Lock::new(*fd).unlock(), 
                   Err(Error::Errno(libc::consts::os::posix88::EBADF)));
    }
}

#[test]
fn lock_ok() {
    let tmp = TempFile::new("file-lock-test");
    for kind in &[LockKind::Blocking, LockKind::NonBlocking] {
        assert_eq!(Lock::new(tmp.fd()).lock(kind.clone(), AccessMode::Write), Ok(()));
    }
}

#[test]
fn unlock_error() {
    let tmp = TempFile::new("file-lock-test");
    for kind in &[LockKind::Blocking, LockKind::NonBlocking] {
        assert_eq!(Lock::new(tmp.fd()).lock(kind.clone(), AccessMode::Write), Ok(()));

        // fcntl() will only allow us to hold a single lock on a file at a time
        // so this test can't work :(
        assert_eq!(Lock::new(tmp.fd()).lock(kind.clone(), AccessMode::Write), Ok(()));


        // unlock without prior lock 
        assert_eq!(Lock::new(tmp.fd()).unlock(), Ok(()));
    }
}

#[test]
fn unlock_ok() {
    let tmp = TempFile::new("file-lock-test");
    for kind in &[LockKind::Blocking, LockKind::NonBlocking] {
        let l = Lock::new(tmp.fd());

        assert_eq!(l.lock(kind.clone(), AccessMode::Write), Ok(()));
        assert_eq!(l.unlock(), Ok(()));
        assert!(l.unlock().is_ok(), "extra unlocks are fine");
    }
}
