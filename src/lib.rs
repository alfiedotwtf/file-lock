//! File locking via POSIX advisory record locks.
//!
//! This crate provides the facility to lock and unlock a file following the
//! advisory record lock scheme as specified by UNIX IEEE Std 1003.1-2001
//! (POSIX.1) via `fcntl()`.
//!
//! # Examples
//!
//! ```
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
//!             InvalidFilename => println!("Invalid filename"),
//!             Errno(i)        => println!("Got filesystem error {}", i),
//!         }
//!     }
//! }
//! ```

extern crate libc;

use std::ffi::CString;
use std::os::unix::io::RawFd;

/// Represents a lock on a file.
#[derive(Debug, Eq, PartialEq)]
pub struct Lock {
  _fd: RawFd,
}

/// Represents the error that occurred while trying to lock or unlock a file.
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
  /// caused when the filename is invalid as it contains a null byte.
  InvalidFilename,
  /// caused when the error occurred at the filesystem layer (see
  /// [errno](https://crates.io/crates/errno)).
  Errno(i32),
}

#[repr(C)]
struct c_result {
  _fd:    RawFd,
  _errno: i32,
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
  fn c_lock(filename: *const libc::c_char, should_block: i32) -> c_result;
  fn c_unlock(fd: i32) -> c_result;
}

impl Lock {


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
    /// ```
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
    ///             InvalidFilename => println!("Invalid filename"),
    ///             Errno(i)        => println!("Got filesystem error {}", i),
    ///         }
    ///     }
    /// }
    /// ```
    pub fn create_file_and_lock(filename: &str, kind: LockKind) -> Result<Lock, Error>{
        match CString::new(filename) {
            Err(_) => Err(Error::InvalidFilename),
            Ok(raw_filename) => {
                let my_result = unsafe { c_lock(raw_filename.as_ptr(), kind.into()) };

                return match my_result._errno {
                   0 => Ok(Lock{_fd: my_result._fd}),
                   _ => Err(Error::Errno(my_result._errno)),
                }
            }
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
    /// ```
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
        let my_result = c_unlock(self._fd);

        return match my_result._errno {
           0 => Ok(()),
           _ => Err(Error::Errno(my_result._errno)),
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
    
    use super::*;
    use super::Error::*;

    //
    // unfortunately we can't abstract this out for lock() and lock_wait()
    // into a macro because string concat doesn't exist
    //

    // lock_wait() tests

    #[test]
    fn lock_invalid_filenames() {
        for kind in &[LockKind::Blocking, LockKind::NonBlocking] {
            assert_eq!(Lock::create_file_and_lock("null\0inside", kind.clone()), 
                       Err(Error::InvalidFilename));
            assert_eq!(Lock::create_file_and_lock("", kind.clone()), 
                       Err(Error::Errno(libc::consts::os::posix88::ENOENT)));
        }
    }

    #[test]
    fn lock_ok() {
        for kind in &[LockKind::Blocking, LockKind::NonBlocking] {
            assert!(Lock::create_file_and_lock("/tmp/file-lock-test", kind.clone()).is_ok());
        }
    }

    #[test]
    fn unlock_error() {
        for kind in &[LockKind::Blocking, LockKind::NonBlocking] {
            let l1 = Lock::create_file_and_lock("/tmp/file-lock-test", kind.clone());
            let l2 = Lock::create_file_and_lock("/tmp/file-lock-test", kind.clone());
        
            assert!(l1.is_ok());
            // fcntl() will only allow us to hold a single lock on a file at a time
            // so this test can't work :(
            assert!(l2.is_ok());
        }
    }
    
    #[test]
    fn unlock_ok() {
        for kind in &[LockKind::Blocking, LockKind::NonBlocking] {
            let l = Lock::create_file_and_lock("/tmp/file-lock-test", kind.clone()).unwrap();
            assert!(l.unlock().is_ok());
            assert_eq!(l.unlock(), Err(Error::Errno(libc::consts::os::posix88::EBADF)));
        }
    }
}
