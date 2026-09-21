//! Real Windows pipe inheritance, isolated from the test harness's own stdio.
use super::*;
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use webcodex_process::ManagedChild;

const TEST_NAME: &str =
    "tests::stdio_inheritance::standard_pipes_do_not_leak_to_unrelated_descendants";
const MODE: &str = "WEBCODEX_STDIO_INHERITANCE_FIXTURE";
const READY: &str = "WEBCODEX_STDIO_INHERITANCE_READY";

fn fixture_command(mode: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args(["--exact", TEST_NAME, "--nocapture"]);
    command.env(MODE, mode);
    command
}

#[test]
fn standard_pipes_do_not_leak_to_unrelated_descendants() {
    match std::env::var(MODE).as_deref() {
        Ok("descendant") => {
            std::fs::write(std::env::var_os(READY).unwrap(), b"ready").unwrap();
            // The outer ManagedChild owns this whole fixture tree. It terminates
            // the still-live descendant only after checking capture EOF.
            std::thread::sleep(Duration::from_secs(60));
            return;
        }
        Ok("explicit-inherit") => {
            println!("EXPLICIT_CHILD_STDOUT");
            eprintln!("EXPLICIT_CHILD_STDERR");
            return;
        }
        Ok("worker") => {
            isolate_inherited_standard_pipes().unwrap();
            println!("PARENT_STDOUT_PRESERVED");
            eprintln!("PARENT_STDERR_PRESERVED");
            let status = fixture_command("explicit-inherit")
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .unwrap();
            assert!(status.success());
            let ready = PathBuf::from(std::env::var_os(READY).unwrap());
            // Deliberately outlive the direct worker. The external ManagedChild
            // owns and reaps both; dropping this Child does not kill it early.
            #[allow(clippy::zombie_processes)]
            let _descendant = fixture_command("descendant")
                .env(READY, &ready)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(15);
            while !ready.is_file() {
                assert!(Instant::now() < deadline, "descendant did not start");
                std::thread::sleep(Duration::from_millis(10));
            }
            return;
        }
        Ok(other) => panic!("unexpected fixture mode: {other}"),
        Err(_) => {}
    }

    let temp = tempfile::tempdir().unwrap();
    let mut command = fixture_command("worker");
    command
        .env(READY, temp.path().join("ready"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut worker = ManagedChild::spawn(&mut command).unwrap();
    let stdout = worker.child_mut().stdout.take().unwrap();
    let stderr = worker.child_mut().stderr.take().unwrap();
    let (tx, rx) = mpsc::channel();
    let stdout_tx = tx.clone();
    let stdout_reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = std::io::BufReader::new(stdout).read_to_end(&mut bytes);
        let _ = stdout_tx.send(("stdout", result, bytes));
    });
    let stderr_reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = std::io::BufReader::new(stderr).read_to_end(&mut bytes);
        let _ = tx.send(("stderr", result, bytes));
    });
    let validation = (|| -> Result<(), String> {
        let deadline = Instant::now() + Duration::from_secs(20);
        let status = loop {
            if let Some(status) = worker.try_wait().map_err(|e| e.to_string())? {
                break status;
            }
            if Instant::now() >= deadline {
                return Err("stdio worker did not exit".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        if !status.success() {
            return Err(format!("stdio worker failed: {status}"));
        }
        if worker.try_tree_exit().map_err(|e| e.to_string())? {
            return Err("descendant exited before the pipe inheritance assertion".into());
        }
        for _ in 0..2 {
            let (stream, read, bytes) = rx
                .recv_timeout(Duration::from_secs(2))
                .map_err(|_| "capture pipe stayed open after the direct worker exited")?;
            read.map_err(|e| e.to_string())?;
            let text = String::from_utf8(bytes).map_err(|e| e.to_string())?;
            let expected = if stream == "stdout" {
                ["PARENT_STDOUT_PRESERVED", "EXPLICIT_CHILD_STDOUT"]
            } else {
                ["PARENT_STDERR_PRESERVED", "EXPLICIT_CHILD_STDERR"]
            };
            if !expected.iter().all(|marker| text.contains(marker)) {
                return Err(format!("explicit {stream} output was lost"));
            }
        }
        Ok(())
    })();
    // Cleanup occurs before asserting the result, including the regression's
    // failure path. No unrelated process or numeric process-tree lookup is used.
    worker.terminate_tree().unwrap();
    assert!(worker.wait_tree_exit(Duration::from_secs(5)).unwrap());
    stdout_reader.join().unwrap();
    stderr_reader.join().unwrap();
    validation.unwrap();
}
