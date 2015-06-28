extern crate file_lock;

mod support;

use std::path::Path;
use std::env;
use std::process::{Command, ExitStatus, Child, Stdio};
use std::thread::sleep_ms;

use file_lock::*;
use support::TempFile;

const ENV_LOCK_FILE: &'static str = "FILE_LOCK_TEST_LOCK_FILE_PATH";
const ENV_LOCK_KIND: &'static str = "FILE_LOCK_TEST_LOCK_KIND";
const ENV_ACCESS_MODE: &'static str = "FILE_LOCK_TEST_ACCESS_MODE";

/// This must be long enough for any testing machine to bring up a process and 
/// execute main.
const WAIT_TIME: u32 = 250;

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
            // Write (exclusive) lock testing
            let t = TempFile::new("inter-process-write-file-lock-operation", 
                                  AccessMode::Write);

            for kind in &[LockKind::NonBlocking, LockKind::Blocking] {

                let mut fl = FileLock::new(t.path_buf(), AccessMode::Write);
                fl.try_lock().unwrap();

                match *kind {
                    LockKind::NonBlocking => {
                        assert!(!exec_self_status(t.path(), kind.clone(), AccessMode::Write)
                                                  .success()
                                , "child can't get exclusive one");
                        assert!(!exec_self_status(t.path(), kind.clone(), AccessMode::Read)
                                                  .success()
                                , "child can't get read lock");

                        fl.unlock().unwrap();
                        assert!(exec_self_status(t.path(), kind.clone(), AccessMode::Write)
                                                  .success()
                                , "child can get exclusive lock");
                        assert!(exec_self_status(t.path(), kind.clone(), AccessMode::Read)
                                                  .success()
                                , "child can get shared lock");
                    },
                    LockKind::Blocking => {
                        let mut child = exec_self_child(t.path(), kind.clone(), 
                                                        AccessMode::Write);
                        assert!(!exec_self_status(t.path(), LockKind::NonBlocking, 
                                                  AccessMode::Write).success(),
                                "can't get non-blocking write lock");
                        sleep_ms(WAIT_TIME);
                        fl.unlock().unwrap();
                        assert!(child.wait().unwrap().success(),
                                "child should get write lock after waiting");

                        fl.lock().unwrap();
                        let mut child = exec_self_child(t.path(), kind.clone(), 
                                                        AccessMode::Read);
                        assert!(!exec_self_status(t.path(), LockKind::NonBlocking, 
                                                  AccessMode::Read).success(),
                                "can't get non-blocking read lock");

                        sleep_ms(WAIT_TIME);
                        fl.unlock().unwrap();

                        assert!(child.wait().unwrap().success(),
                                "child should get read lock after waiting");
                    }
                } 
            }// end for each lock kind

            let t = TempFile::new("inter-process-read-file-lock-operation", 
                                  AccessMode::Read);

            let mut fl = FileLock::new(t.path_buf(), AccessMode::Read);
            fl.try_lock().unwrap();

            for kind in &[LockKind::NonBlocking, LockKind::Blocking] {
                assert!(exec_self_status(t.path(), kind.clone(), AccessMode::Read)
                                        .success()
                        , "child can get shared lock");
            }

            let mut child = exec_self_child(t.path(), LockKind::Blocking, AccessMode::Write);
            assert!(!exec_self_status(t.path(), LockKind::NonBlocking, AccessMode::Write)
                                      .success(),
                    "Cannot obtain exclusie lock while there is a reader");

            sleep_ms(WAIT_TIME);
            fl.unlock().unwrap();

            assert!(child.wait().unwrap().success(),
                    "child can get exclusive lock as we dropped the shared one");
        }
    }
}
