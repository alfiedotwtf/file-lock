extern crate file_lock;

mod support;

use std::env;
use std::process::{Command, ExitStatus};
use std::os::unix::io::AsRawFd;
use std::fs::OpenOptions;

use file_lock::*;
use support::TempFile;


const ENV_LOCK_FILE: &'static str = "LOCK_TEST_LOCK_FILE_PATH";

#[test]
fn inter_process_lock() {
    match env::var(ENV_LOCK_FILE) {
        Ok(path) => {
            let file = OpenOptions::new()
                                   .write(true)
                                   .open(&path).unwrap();

            // we are the slave and just attempt to acquire a lock.
            Lock::new(file.as_raw_fd())
                 .lock(LockKind::NonBlocking, AccessMode::Write).unwrap();
        },
        Err(_) => {
            // we are the driver
            let t = TempFile::new("inter-process-write-lock-operation", AccessMode::Write);

            let exec_self_status = || -> ExitStatus {
                Command::new(env::current_exe().unwrap())
                        .env(ENV_LOCK_FILE, t.path())
                        .output().unwrap().status
            };
            
            {
                let l = Lock::new(t.fd());
                l.lock(LockKind::NonBlocking, AccessMode::Write).unwrap();
                assert!(!exec_self_status().success(), "Other process can't take lock");
            }

            assert!(exec_self_status().success(), "Now it can have the lock");
        }
    }
}
