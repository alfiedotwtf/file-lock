use std::os::unix::io::RawFd;
use std::fmt;
use std::error::Error as ErrorTrait;
use errno;
use libc::c_int;
pub use util::{Kind, Mode, ParseError};

extern {
    fn c_lock(fd: i32, should_block: i32, is_write_lock: i32) -> c_int;
    fn c_unlock(fd: i32) -> c_int;
}


/// Represents a write lock on a file.
///
/// The `lock(Kind)` method tries to obtain a write-lock on the
/// file identified by a file-descriptor. 
/// One can obtain different kinds of write-locks.
///
/// * Kind::NonBlocking - immediately return with an `Errno` error.
/// * Kind::Blocking - waits (i.e. blocks the running thread) for the current
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
/// use file_lock::fd::{Lock, Error, Mode, Kind};
/// use std::os::unix::io::AsRawFd;
///
/// fn main() {
///     let f = tempfile::TempFile::new().unwrap();
///
///     match Lock::new(f.as_raw_fd()).lock(Kind::NonBlocking, Mode::Write) {
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
    Errno(errno::Errno),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::Errno(ref errno)
                => write!(f, "Lock operation failed: {}", errno)
        }
    }
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Errno(_) 
                => "Failed to acuire file lock",
        }
    }
}

/// Obtain a write-lock the file-descriptor
/// 
/// For an example, please see the documentation of the [`Lock`](struct.Lock.html) structure.
pub fn lock(fd: RawFd, kind: Kind, mode: Mode) -> Result<(), Error> {
    let errno = unsafe { c_lock(fd, kind.into(), mode.into()) };

    return match errno {
       0 => Ok(()),
       _ => Err(Error::Errno(errno::Errno(errno))),
    }
}

/// Unlocks the file held by `Lock`.
///
/// In reality, you shouldn't need to call `unlock()`. As `Lock` implements
/// the `Drop` trait, once the `Lock` reference goes out of scope, `unlock()`
/// will be called automatically.
///
/// For an example, please see the documentation of the [`Lock`](struct.Lock.html) structure.
pub fn unlock(fd: RawFd) -> Result<(), Error> {
  unsafe {
    let errno = c_unlock(fd);

    return match errno {
       0 => Ok(()),
       _ => Err(Error::Errno(errno::Errno(errno))),
    }
  }
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
    pub fn lock(&self, kind: Kind, mode: Mode) -> Result<(), Error> {
        lock(self.fd, kind.clone(), mode.clone())
    }

    /// Unlocks the file held by `Lock`.
    ///
    /// In reality, you shouldn't need to call `unlock()`. As `Lock` implements
    /// the `Drop` trait, once the `Lock` reference goes out of scope, `unlock()`
    /// will be called automatically.
    ///
    /// For an example, please see the documentation of the [`Lock`](struct.Lock.html) structure.
    pub fn unlock(&self) -> Result<(), Error> {
        unlock(self.fd)
    }
}

#[allow(unused_must_use)]
impl Drop for Lock {
    fn drop(&mut self) {
        self.unlock().ok();
    }
}
