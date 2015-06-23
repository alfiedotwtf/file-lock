# NAME

file-lock - File locking via POSIX advisory record locks

This crate provides the facility to lock and unlock a file following the
advisory record lock scheme as specified by UNIX IEEE Std 1003.1-2001 (POSIX.1)
via fcntl().

# USAGE

    extern crate file_lock;

    use file_lock::*;
    use file_lock::Error::*;
    
    fn main() {
      let l = lock("/tmp/file-lock-test");
    
      match l {
          Ok(_)  => println!("Got lock"),
          Err(e) => match e {
            InvalidFilename => println!("Invalid filename"),
            Errno(i)        => println!("Got filesystem error {}", i),
          }
      }
    }

# DOCUMENTATION

* [https://alfiedotwtf.github.io/file-lock/][https://alfiedotwtf.github.io/file-lock/]

# SUPPORT

[![Build Status](https://travis-ci.org/alfiedotwtf/file-lock.svg?branch=idiomatic-rust)](https://travis-ci.org/alfiedotwtf/file-lock)

Please report any bugs or feature requests at:

* [https://github.com/alfiedotwtf/file-lock/issues](https://github.com/alfiedotwtf/file-lock/issues)

Watch the repository and keep up with the latest changes:

* [https://github.com/alfiedotwtf/file-lock/subscription](https://github.com/alfiedotwtf/file-lock/subscription)

Feel free to fork the repository and submit pull requests :)

# DEPENDENCIES

* [gcc](https://gcc.gnu.org/)

# SEE ALSO

* [Lock, Stock and Two Smoking Barrels](http://www.imdb.com/title/tt0120735/)

# AUTHOR

[Alfie John](https://www.alfie.wtf) &lt;[alfie@alfie.wtf](mailto:alfie@alfie.wtf)&gt;

# WARRANTY

IT COMES WITHOUT WARRANTY OF ANY KIND.

# COPYRIGHT AND LICENSE

MIT License

Copyright (c) 2015 Alfie John

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
