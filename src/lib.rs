//! File locking via POSIX advisory record locks.
//!
//! This crate provides the facility to lock and unlock a file following the
//! advisory record lock scheme as specified by UNIX IEEE Std 1003.1-2001
//! (POSIX.1) via `fcntl()`.
//!
//! # Examples
//!
//! ```ignore
//! extern crate file_lock;
//!
//! use file_lock::*;
//! use file_lock::Error::*;
//!
//! fn main() {
//!     let l = Lock::create_file_and_lock("/tmp/file-lock-test", LockKind::NonBlocking);
//!
//!     match l {
//!         Ok(_)  => println!("Got lock"),
//!         Err(e) => match e {
//!             Errno(i)        => println!("Got filesystem error {}", i),
//!         }
//!     }
//! }
//! ```

extern crate libc;

use std::os::unix::io::RawFd;

/// Represents a lock on a file.
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

#[derive(Clone, Debug)]
pub enum LockKind {
    /// Indicates a lock file which 
    NonBlocking,
    Blocking,
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
  fn c_lock(fd: i32, should_block: i32) -> i32;
  fn c_unlock(fd: i32) -> i32;
}

impl Lock {
    /// Create a new lock instance from the given file descriptor `fd`.
    /// 
    /// You will have to call `lock()` on it to acquire any lock.
    ///
    // TODO(ST): doc update once API has settled
    /// Locks the specified file.
    ///
    /// The `lock()` and `lock_wait()` functions try to perform a lock on the
    /// specified file. The difference between the two is what they do when
    /// another process has a lock on the same file:
    ///
    /// * lock() - immediately return with an `Errno` error.
    /// * lock_wait() - waits (i.e. blocks the running thread) for the current
    /// owner of the lock to relinquish the lock.
    ///
    /// # Example
    ///
    /// ```ignore
    /// extern crate file_lock;
    ///
    /// use file_lock::*;
    /// use file_lock::Error::*;
    ///
    /// fn main() {
    ///     let l = Lock::create_file_and_lock("/tmp/file-lock-test", LockKind::NonBlocking);
    ///
    ///     match l {
    ///         Ok(_)  => println!("Got lock"),
    ///         Err(e) => match e {
    ///             Errno(i)        => println!("Got filesystem error {}", i),
    ///         }
    ///     }
    /// }
    /// ```
    pub fn new(fd: RawFd) -> Lock {
        Lock {
            fd:   fd,
        }
    }

    /// Lock the file-descriptor 
    pub fn lock(&self, kind: LockKind) -> Result<(), Error> {
        let errno = unsafe { c_lock(self.fd, kind.into()) };

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
    /// # Example
    ///
    /// ```ignore
    /// extern crate file_lock;
    ///
    /// use file_lock::*;
    ///
    /// fn main() {
    ///     let l = Lock::create_file_and_lock("/tmp/file-lock-test", LockKind::NonBlocking).unwrap();
    ///
    ///     if l.unlock().is_ok() {
    ///         println!("Unlocked!");
    ///     }
    /// }
    /// ```
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
                assert_eq!(Lock::new(*fd).lock(kind.clone()), 
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
            assert_eq!(Lock::new(tmp.fd()).lock(kind.clone()), Ok(()));
        }
    }

    #[test]
    fn unlock_error() {
        let tmp = TempFile::new("file-lock-test");
        for kind in &[LockKind::Blocking, LockKind::NonBlocking] {
            assert_eq!(Lock::new(tmp.fd()).lock(kind.clone()), Ok(()));

            // fcntl() will only allow us to hold a single lock on a file at a time
            // so this test can't work :(
            assert_eq!(Lock::new(tmp.fd()).lock(kind.clone()), Ok(()));


            // unlock without prior lock 
            assert_eq!(Lock::new(tmp.fd()).unlock(), Ok(()));
        }
    }
    
    #[test]
    fn unlock_ok() {
        let tmp = TempFile::new("file-lock-test");
        for kind in &[LockKind::Blocking, LockKind::NonBlocking] {
            let l = Lock::new(tmp.fd());

            assert_eq!(l.lock(kind.clone()), Ok(()));
            assert_eq!(l.unlock(), Ok(()));
            assert!(l.unlock().is_ok(), "extra unlocks are fine");
        }
    }
}
