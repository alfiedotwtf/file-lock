#![feature(libc)]
extern crate libc;

use std::ffi::CString;

pub struct Lock {
  _fd: i32,
}

pub enum Error {
  InvalidFilename,
  Errno(i32),
}

#[repr(C)]
struct c_result {
  _fd:    i32,
  _errno: i32,
}

macro_rules! _create_lock_type {
  ($lock_type:ident, $c_lock_type:ident) => {
    extern {
      fn $c_lock_type(filename: *const libc::c_char) -> c_result;
    }

    pub fn $lock_type(filename: &str) -> Result<Lock, Error> {
      let raw_filename = CString::new(filename);

      if raw_filename.is_err() {
        return Err(Error::InvalidFilename);
      }

      unsafe {
        let my_result = $c_lock_type(raw_filename.unwrap().as_ptr());

        return match my_result._fd {
          -1 => Err(Error::Errno(my_result._errno)),
           _ => Ok(Lock{_fd: my_result._fd}),
        }
      }
    }
  };
}

_create_lock_type!(lock, c_lock);
_create_lock_type!(lock_wait, c_lock_wait);

extern {
  fn c_unlock(_fd: i32) -> c_result;
}

pub fn unlock(lock: &Lock) -> Result<bool, Error> {
  unsafe {
    let my_result = c_unlock(lock._fd);

    return match my_result._fd {
      -1 => Err(Error::Errno(my_result._errno)),
       _ => Ok(true),
    }
  }
}

#[allow(unused_must_use)]
impl Drop for Lock {
  fn drop(&mut self) {
    unlock(self);
  }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::Error::*;

    //
    // unfortunately we can't abstract this out for lock() and lock_wait()
    // into a macro because string concat doesn't exist
    //

    // lock_wait() tests

    #[test]
    fn lock_invalid_filename() {
        assert_eq!(_lock("null\0inside"), "invalid");
    }

    #[test]
    fn lock_errno() {
        assert_eq!(_lock(""), "errno");
    }

    #[test]
    fn lock_ok() {
        assert_eq!(_lock("/tmp/file-lock-test"), "ok");
    }

    fn _lock(filename: &str) -> &str {
        let l = lock(filename);

        match l {
            Ok(_)  => "ok",
            Err(e) => match e {
                InvalidFilename => "invalid",
                Errno(_)        => "errno",
            }
        }
    }

    // lock_wait() tests

    #[test]
    fn lock_wait_invalid_filename() {
        assert_eq!(_lock_wait("null\0inside"), "invalid");
    }

    #[test]
    fn lock_wait_errno() {
        assert_eq!(_lock_wait(""), "errno");
    }

    #[test]
    fn lock_wait_ok() {
        assert_eq!(_lock_wait("/tmp/file-lock-test"), "ok");
    }

    fn _lock_wait(filename: &str) -> &str {
        let l = lock_wait(filename);

        match l {
            Ok(_)  => "ok",
            Err(e) => match e {
                InvalidFilename => "invalid",
                Errno(_)        => "errno",
            }
        }
    }

    // unlock()

    //
    // fcntl() will only allow us to hold a single lock on a file at a time
    // so this test can't work :(
    //
    // #[test]
    // fn unlock_error() {
    //     let l1 = lock("/tmp/file-lock-test");
    //     let l2 = lock("/tmp/file-lock-test");
    //
    //     assert!(l1.is_ok());
    //     assert!(l2.is_err());
    // }
    //

    #[test]
    fn unlock_ok() {
        let l        = lock_wait("/tmp/file-lock-test");
        let unlocked = l.ok().unwrap();

        assert!(unlock(&unlocked).is_ok(), true);
    }
}
