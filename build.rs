extern crate gcc;

fn main() {
  gcc::compile_library("liblock.a", &["src/lock.c"]);
}
