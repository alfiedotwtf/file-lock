extern crate file_lock;
extern crate libc;
extern crate errno;

mod support;

use std::os::unix::io::RawFd;
use std::env;
use std::fs;

use support::{TempFile, Remover};
use file_lock::*;

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
                       Err(Error::Errno(errno::Errno(libc::consts::os::posix88::EBADF))));
        }

        assert_eq!(Lock::new(*fd).unlock(), 
                   Err(Error::Errno(errno::Errno(libc::consts::os::posix88::EBADF))));
    }
}

#[test]
fn lock_ok() {
    let tmp = TempFile::new("file-lock-test", AccessMode::Write);
    for kind in &[LockKind::Blocking, LockKind::NonBlocking] {
        assert_eq!(Lock::new(tmp.fd()).lock(kind.clone(), AccessMode::Write), Ok(()));
    }
}

#[test]
fn unlock_error() {
    let tmp = TempFile::new("file-lock-test", AccessMode::Write);
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
    let tmp = TempFile::new("file-lock-test", AccessMode::Write);
    for kind in &[LockKind::Blocking, LockKind::NonBlocking] {
        let l = Lock::new(tmp.fd());

        assert_eq!(l.lock(kind.clone(), AccessMode::Write), Ok(()));
        assert_eq!(l.unlock(), Ok(()));
        assert!(l.unlock().is_ok(), "extra unlocks are fine");
    }
}

#[test]
fn file_lock_create_file() {
    use std::io::Write;

    let mut path = env::temp_dir();
    path.push("file-lock-creation-test");

    let _r = {
        let mut fl = FileLock::new(path.clone(), AccessMode::Write);
        let r = Remover { path: fl.path().clone() };
        fl.lock().unwrap();

        fl.file().unwrap().write(b"hello").unwrap();

        assert!(fs::metadata(&path).is_ok(), "File should have been created");
        fl.unlock().unwrap();
        assert!(fs::metadata(&path).is_ok(), "File is still there after unlock");
        r
    };

    assert!(fs::metadata(&path).is_ok(), "File is still there after dropping FileLock instance");
}
