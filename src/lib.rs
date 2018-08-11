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
//!
//! use file_lock::FileLock;
//! use std::io::prelude::*;
//!
//! fn main() {
//!     let should_we_block  = true;
//!     let lock_for_writing = true;
//!
//!     let mut filelock = match FileLock::lock("myfile.txt", should_we_block, lock_for_writing) {
//!         Ok(lock) => lock,
//!         Err(err) => panic!("Error getting write lock: {}", err),
//!     };
//!
//!     filelock.file.write_all(b"Hello, World!").is_ok();
//!
//!     // Manually unlocking is optional as we unlock on Drop
//!     filelock.unlock();
//! }
//! ```

extern crate libc;
extern crate nix;

use libc::c_int;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Error;
use std::os::unix::io::AsRawFd;

extern "C" {
    fn c_lock(fd: i32, is_blocking: i32, is_writeable: i32) -> c_int;
    fn c_unlock(fd: i32) -> c_int;
}

/// Represents the actually locked file
#[derive(Debug)]
pub struct FileLock {
    /// the `std::fs::File` of the file that's locked
    pub file: File,
}

impl FileLock {
    /// Try to lock the specified file
    ///
    /// # Parameters
    ///
    /// `filename` is the path of the file we want to lock on
    ///
    /// `is_blocking` is a flag to indicate if we should block if it's already locked
    ///
    /// `is_writable` is a flag to indicate if we want to lock for writing
    ///
    /// # Examples
    ///
    ///```
    ///extern crate file_lock;
    ///
    ///use file_lock::FileLock;
    ///use std::io::prelude::*;
    ///
    ///fn main() {
    ///    let should_we_block  = true;
    ///    let lock_for_writing = true;
    ///
    ///    let mut filelock = match FileLock::lock("myfile.txt", should_we_block, lock_for_writing) {
    ///        Ok(lock) => lock,
    ///        Err(err) => panic!("Error getting write lock: {}", err),
    ///    };
    ///
    ///    filelock.file.write_all(b"Hello, World!").is_ok();
    ///}
    ///```
    ///
    pub fn lock(filename: &str, is_blocking: bool, is_writable: bool) -> Result<FileLock, Error> {
        let file = OpenOptions::new()
            .read(!is_writable)
            .write(is_writable)
            .create(is_writable)
            .open(&filename);

        match file {
            Err(err) => Err(err),
            Ok(file) => {
                let errno = unsafe {
                    c_lock(file.as_raw_fd(), is_blocking as i32, is_writable as i32)
                };

                match errno {
                    0 => Ok(FileLock { file }),
                    _ => Err(Error::from_raw_os_error(errno)),
                }
            },
        }
    }

    /// Unlock our locked file
    ///
    /// *Note:* This method is optional as the file lock will be unlocked automatically when dropped
    ///
    /// # Examples
    ///
    ///```
    ///extern crate file_lock;
    ///
    ///use file_lock::FileLock;
    ///use std::io::prelude::*;
    ///
    ///fn main() {
    ///    let should_we_block  = true;
    ///    let lock_for_writing = true;
    ///
    ///    let mut filelock = match FileLock::lock("myfile.txt", should_we_block, lock_for_writing) {
    ///        Ok(lock) => lock,
    ///        Err(err) => panic!("Error getting write lock: {}", err),
    ///    };
    ///
    ///    filelock.file.write_all(b"Hello, World!").is_ok();
    ///
    ///    match filelock.unlock() {
    ///        Ok(_)    => println!("Successfully unlocked the file"),
    ///        Err(err) => panic!("Error unlocking the file: {}", err),
    ///    };
    ///}
    ///```
    ///
    pub fn unlock(&self) -> Result<(), Error> {
        let errno = unsafe {
            c_unlock(self.file.as_raw_fd())
        };

        match errno {
            0 => Ok(()),
            _ => Err(Error::from_raw_os_error(errno)),
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        self.unlock().is_ok();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use nix::unistd::ForkResult::{Parent, Child};
    use nix::unistd::fork;
    use std::fs::remove_file;
    use std::process;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn lock_and_unlock() {
        let filename = "filelock.test";

        for already_exists in &[true, false] {
            for already_locked in &[true, false] {
                for already_writable in &[true, false] {
                    for is_blocking in &[true, false] {
                        for is_writable in &[true, false] {
                            if !*already_exists && (*already_locked || *already_writable) {
                                // nonsensical tests
                                continue;
                            }

                            remove_file(&filename).is_ok();

                            let parent_lock = match *already_exists {
                                false => None,
                                true  => {
                                    OpenOptions::new()
                                        .write(true)
                                        .create(true)
                                        .open(&filename)
                                        .is_ok();

                                    match *already_locked {
                                        false => None,
                                        true  => match FileLock::lock(&filename, true, *already_writable) {
                                            Ok(lock) => Some(lock),
                                            Err(err) => panic!("Error creating parent lock ({})", err),
                                        },
                                    }
                                },
                            };

                            match fork() {
                                Ok(Parent { child: _ }) => {
                                    sleep(Duration::from_millis(150));

                                    match parent_lock {
                                        Some(lock) => { lock.unlock().is_ok(); },
                                        None       => {},
                                    }

                                    sleep(Duration::from_millis(350));
                                }
                                Ok(Child) => {
                                    let mut try_count = 0;
                                    let mut locked    = false;

                                    match *already_locked {
                                        true => match *is_blocking {
                                            true => {
                                                match FileLock::lock(filename, *is_blocking, *is_writable) {
                                                    Ok(_)  => { locked = true },
                                                    Err(_) => panic!("Error getting lock after wating for release"),
                                                }
                                            },
                                            false => {
                                                for _ in 0..5 {
                                                    match FileLock::lock(filename, *is_blocking, *is_writable) {
                                                        Ok(_) => {
                                                            locked = true;
                                                            break;
                                                        },
                                                        Err(_) => {
                                                            sleep(Duration::from_millis(50));
                                                            try_count = try_count + 1;
                                                        },
                                                    }
                                                }
                                            },
                                        },
                                        false => match FileLock::lock(filename, *is_blocking, *is_writable) {
                                            Ok(_)  => { locked = true },
                                            Err(_) => match !*already_exists && !*is_writable {
                                                true  => {},
                                                false => panic!("Error getting lock with no competition"),
                                            },
                                        },
                                    }

                                    match !*already_exists && !is_writable {
                                        true  => assert!(locked == false, "Locking a non-existent file for reading should fail"),
                                        false => assert!(locked == true, "Lock should have been successful"),
                                    }

                                    match *is_blocking {
                                        true  => assert!(try_count == 0, "Try count should be zero when blocking"),
                                        false => {
                                            match *already_locked {
                                                false => assert!(try_count == 0, "Try count should be zero when no competition"),
                                                true  => match !*already_writable && !is_writable {
                                                    true  => assert!(try_count == 0, "Read lock when locked for reading should succeed first go"),
                                                    false => assert!(try_count >= 3, "Try count should be >= 3"),
                                                },
                                            }
                                        },
                                    }

                                    process::exit(7);
                                },
                                Err(_) => {
                                    panic!("Error forking tests :(");
                                }
                            }

                            remove_file(&filename).is_ok();
                        }
                    }
                }
            }
        }
    }
}
