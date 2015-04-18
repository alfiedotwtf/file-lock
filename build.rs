use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
  let out_dir = env::var("OUT_DIR").unwrap();

  Command::new("gcc")
    .args(&["src/lock.c", "-c", "-fPIC", "-o"])
    .arg(&format!("{}/lock.o", out_dir))
    .status().unwrap();

  Command::new("ar")
    .args(&["crus", "liblock.a", "lock.o"])
    .current_dir(&Path::new(&out_dir))
    .status().unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=lock");
}
