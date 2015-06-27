extern crate file_lock;

mod support;

use std::path::{PathBuf, Path};
use std::env;
use std::process::{Command, ExitStatus, Child, Stdio};

use file_lock::*;
use support::TempFile;

const ENV_LOCK_FILE: &'static str = "FILE_LOCK_TEST_LOCK_FILE_PATH";
const ENV_LOCK_KIND: &'static str = "FILE_LOCK_TEST_LOCK_KIND";
const ENV_ACCESS_MODE: &'static str = "FILE_LOCK_TEST_ACCESS_MODE";

fn configure_command(mut cmd: Command, path: &Path, kind: LockKind, mode: AccessMode)
                                                            -> Command {
    cmd.env(ENV_LOCK_FILE, path)
       .env(ENV_LOCK_KIND, kind.as_ref())
       .env(ENV_ACCESS_MODE, mode.as_ref());
    cmd
}

fn exec_self_child(path: &Path, kind: LockKind, mode: AccessMode) -> Child {
    configure_command(Command::new(env::current_exe().unwrap()), path, kind, mode)
               .stdin(Stdio::null())
               .stdout(Stdio::null())
               .stderr(Stdio::null())
               .spawn().unwrap()
}

fn exec_self_status(path: &Path, kind: LockKind, mode: AccessMode) -> ExitStatus {
    configure_command(Command::new(env::current_exe().unwrap()), path, kind, mode)
               .output().unwrap().status
}

#[test]
fn inter_process_file_lock() {
    match env::var(ENV_LOCK_FILE) {
        Ok(path) => {
            let kind: LockKind = env::var(ENV_LOCK_KIND).unwrap().parse().unwrap();
            let mode: AccessMode = env::var(ENV_ACCESS_MODE).unwrap().parse().unwrap();

            FileLock::new(path.into(), mode).any_lock(kind).unwrap();
        },
        Err(_) => {
            {
                let t = TempFile::new("inter-process-write-file-lock-operation", 
                                      AccessMode::Write);

                for kind in &[LockKind::NonBlocking] {

                    let fl = FileLock::new(t.path_buf(), AccessMode::Write);
                    fl.try_lock().unwrap();

                    if *kind == LockKind::NonBlocking {
                        assert!(!exec_self_status(t.path(), kind.clone(), AccessMode::Write)
                                                  .success())
                    }
                }
            }
        }
    }
}
