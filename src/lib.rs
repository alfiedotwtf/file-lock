//! File locking via POSIX advisory record locks.
//!
//! This crate provides the facility to obtain a write-lock and unlock a file 
//! following the advisory record lock scheme as specified by UNIX IEEE Std 1003.1-2001
//! (POSIX.1) via `fcntl()`.
//!
//! # Examples
//!
//! Please note that the examples use `tempfile` merely to quickly create a file
//! which is removed automatically. In the common case, you would want to lock
//! a file which is known to multiple processes.
//!
//! ```
//! extern crate file_lock;
//! extern crate tempfile;
//!
//! use file_lock::*;
//! use std::os::unix::io::AsRawFd;
//!
//! fn main() {
//!     let f = tempfile::TempFile::new().unwrap();
//!
//!     match Lock::new(f.as_raw_fd()).lock(LockKind::NonBlocking, AccessMode::Write) {
//!         Ok(_)  => println!("Got lock"),
//!         Err(Error::Errno(i))
//!               => println!("Got filesystem error {}", i),
//!     }
//! }
//! ```

extern crate libc;

use std::os::unix::io::RawFd;

/// Represents a write lock on a file.
///
/// The `lock(LockKind)` method tries to obtain a write-lock on the
/// file identified by a file-descriptor. 
/// One can obtain different kinds of write-locks.
///
/// * LockKind::NonBlocking - immediately return with an `Errno` error.
/// * LockKind::Blocking - waits (i.e. blocks the running thread) for the current
/// owner of the lock to relinquish the lock.
///
/// # Example
///
/// Please note that the examples use `tempfile` merely to quickly create a file
/// which is removed automatically. In the common case, you would want to lock
/// a file which is known to multiple processes.
///
/// ```
/// extern crate file_lock;
/// extern crate tempfile;
///
/// use file_lock::*;
/// use std::os::unix::io::AsRawFd;
///
/// fn main() {
///     let f = tempfile::TempFile::new().unwrap();
///
///     match Lock::new(f.as_raw_fd()).lock(LockKind::NonBlocking, AccessMode::Write) {
///         Ok(_)  => {
///             // we have a lock, which is discarded automatically. Otherwise you could call
///             // `unlock()` to make it explicit
///             // 
///             println!("Got lock");
///         },
///         Err(Error::Errno(i))
///               => println!("Got filesystem error {}", i),
///     }
/// }
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct Lock {
  fd: RawFd,
}

/// Represents the error that occurred while trying to lock or unlock a file.
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
  /// caused when the error occurred at the filesystem layer (see
  /// [errno](https://crates.io/crates/errno)).
  Errno(i32),
}

/// Represents the kind of lock (e.g. *blocking*, *non-blocking*)
#[derive(Clone, Debug)]
pub enum LockKind {
    /// Indicates a lock file which 
    NonBlocking,
    Blocking,
}

/// Represents a file access mode, e.g. read or write
#[derive(Clone, Debug)]
pub enum AccessMode {
    Read,
    Write
}

impl Into<i32> for AccessMode {
    fn into(self) -> i32 {
        match self {
            AccessMode::Read => 0,
            AccessMode::Write => 1,
        }
    }
}

impl Into<i32> for LockKind {
    fn into(self) -> i32 {
        match self {
            LockKind::NonBlocking => 0,
            LockKind::Blocking => 1,
        }
    }
}

extern {
  fn c_lock(fd: i32, should_block: i32, is_write_lock: i32) -> i32;
  fn c_unlock(fd: i32) -> i32;
}

impl Lock {
    /// Create a new lock instance from the given file descriptor `fd`.
    /// 
    /// You will have to call `lock(...)` on it to acquire any lock.
    pub fn new(fd: RawFd) -> Lock {
        Lock {
            fd:   fd,
        }
    }

    /// Obtain a write-lock the file-descriptor
    /// 
    /// For an example, please see the documentation of the [`Lock`](struct.Lock.html) structure.
    pub fn lock(&self, kind: LockKind, mode: AccessMode) -> Result<(), Error> {
        let errno = unsafe { c_lock(self.fd, kind.into(), mode.into()) };

        return match errno {
           0 => Ok(()),
           _ => Err(Error::Errno(errno)),
        }
    }

    /// Unlocks the file held by `Lock`.
    ///
    /// In reality, you shouldn't need to call `unlock()`. As `Lock` implements
    /// the `Drop` trait, once the `Lock` reference goes out of scope, `unlock()`
    /// will be called automatically.
    ///
    /// For an example, please see the documentation of the [`Lock`](struct.Lock.html) structure.
    pub fn unlock(&self) -> Result<(), Error> {
      unsafe {
        let errno = c_unlock(self.fd);

        return match errno {
           0 => Ok(()),
           _ => Err(Error::Errno(errno)),
        }
      }
    }
}

#[allow(unused_must_use)]
impl Drop for Lock {
  fn drop(&mut self) {
    self.unlock().ok();
  }
}

#[cfg(test)]
mod test {
    use libc;

    use std::env;
    use std::fs::{File, OpenOptions, remove_file};
    use std::path::PathBuf;
    use std::os::unix::io::{RawFd, AsRawFd};

    use super::*;

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
}
