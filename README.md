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

# SUPPORT

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

Copyright (C) 2015 by Alfie John

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with
this program. If not, see [http://www.gnu.org/licenses/](http://www.gnu.org/licenses/).
