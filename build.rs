extern crate gcc;

fn main() {
  gcc::Build::new()
    .file("src/file_lock.c")
    .compile("libfile_lock.a")
}
