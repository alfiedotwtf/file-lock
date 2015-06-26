use std::path::PathBuf;
use std::fs::remove_file;
use std::borrow::Borrow;

/// A utility type to assure the removal of a file.
///
/// It is useful when a temporary lock file is created. When an instance dropped
/// of this type is dropped, the lock file will be removed. It is not an error
/// if the file doesn't exist anymore.
pub struct Remover<P: Borrow<PathBuf>> {
  pub path: P,
}

impl<P> Drop for Remover<P> 
    where P: Borrow<PathBuf> {
    fn drop(&mut self) {
        remove_file(self.path.borrow()).ok();
    }
}

