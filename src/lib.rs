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

        return match my_result._errno {
           0 => Ok(Lock{_fd: my_result._fd}),
          -1 => Err(Error::Errno(my_result._errno)),
           _ => unreachable!(),
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

    return match my_result._errno {
       0 => Ok(true),
      -1 => Err(Error::Errno(my_result._errno)),
       _ => unreachable!(),
    }
  }
}

#[allow(unused_must_use)]
impl Drop for Lock {
  fn drop(&mut self) {
    unlock(self);
  }
}
