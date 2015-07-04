//#![doc(html_root_url = "https://alfiedotwtf.github.io/file-lock/")]

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
//! use file_lock::fd::{Lock, Error, Mode, Kind};
//! use std::os::unix::io::AsRawFd;
//!
//! fn main() {
//!     let f = tempfile::TempFile::new().unwrap();
//!
//!     match Lock::new(f.as_raw_fd()).lock(Kind::NonBlocking, Mode::Write) {
//!         Ok(_)  => println!("Got lock"),
//!         Err(Error::Errno(i))
//!               => println!("Got filesystem error {}", i),
//!     }
//! }
//! ```

extern crate libc;
extern crate errno;

pub mod fd;
pub mod filename;
mod util;