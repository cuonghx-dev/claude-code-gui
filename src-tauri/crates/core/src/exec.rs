//! Run a user-supplied shell command with a timeout and output caps.
//!
//! Only used where a person explicitly asks for it — the statusline "Test run"
//! button today. Nothing in this crate runs a configured command on its own:
//! statusline commands and hooks are arbitrary code, and this app configures
//! them rather than executing them.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::AppError;

/// Captured output is capped at this many bytes per stream.
pub const MAX_OUTPUT_BYTES: usize = 8 * 1024;

#[derive(Debug, Clone)]
pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub timed_out: bool,
    pub truncated: bool,
}

/// Run `cmd` through the platform shell, feed it `stdin`, and give up after
/// `timeout`.
pub fn run_with_timeout(
    cmd: &str,
    cwd: &Path,
    stdin: &str,
    timeout: Duration,
) -> Result<ExecResult, AppError> {
    let started = Instant::now();

    #[cfg(target_os = "windows")]
    let mut child = Command::new("cmd")
        .args(["/C", cmd])
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    #[cfg(not(target_os = "windows"))]
    let mut child = Command::new("sh")
        .args(["-c", cmd])
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut pipe) = child.stdin.take() {
        // A command that ignores stdin closes the pipe early; that is a broken
        // pipe, not a failure worth reporting.
        let _ = pipe.write_all(stdin.as_bytes());
    }

    let mut timed_out = false;
    loop {
        match child.try_wait()? {
            Some(_) => break,
            None if started.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                timed_out = true;
                break;
            }
            None => std::thread::sleep(Duration::from_millis(20)),
        }
    }

    let output = child.wait_with_output()?;
    let (stdout, out_truncated) = cap(&output.stdout);
    let (stderr, err_truncated) = cap(&output.stderr);

    Ok(ExecResult {
        stdout,
        stderr,
        exit_code: output.status.code(),
        duration_ms: started.elapsed().as_millis() as u64,
        timed_out,
        truncated: out_truncated || err_truncated,
    })
}

fn cap(bytes: &[u8]) -> (String, bool) {
    let truncated = bytes.len() > MAX_OUTPUT_BYTES;
    let slice = &bytes[..bytes.len().min(MAX_OUTPUT_BYTES)];
    (String::from_utf8_lossy(slice).into_owned(), truncated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_stdout_and_exit_code() {
        let td = tempfile::tempdir().unwrap();
        let r = run_with_timeout("echo hello", td.path(), "", Duration::from_secs(5)).unwrap();
        assert_eq!(r.stdout.trim(), "hello");
        assert_eq!(r.exit_code, Some(0));
        assert!(!r.timed_out && !r.truncated);
    }

    #[test]
    fn stdin_reaches_the_command() {
        let td = tempfile::tempdir().unwrap();
        let r = run_with_timeout("cat", td.path(), "{\"a\":1}", Duration::from_secs(5)).unwrap();
        assert_eq!(r.stdout, "{\"a\":1}");
    }

    #[test]
    fn a_nonzero_exit_is_reported_not_raised() {
        let td = tempfile::tempdir().unwrap();
        let r = run_with_timeout("echo oops >&2; exit 3", td.path(), "", Duration::from_secs(5))
            .unwrap();
        assert_eq!(r.exit_code, Some(3));
        assert_eq!(r.stderr.trim(), "oops");
    }

    #[test]
    fn a_hanging_command_is_killed() {
        let td = tempfile::tempdir().unwrap();
        let r = run_with_timeout("sleep 10", td.path(), "", Duration::from_millis(200)).unwrap();
        assert!(r.timed_out);
        assert!(r.duration_ms < 5_000, "the watchdog reaped it: {}ms", r.duration_ms);
    }

    #[test]
    fn oversize_output_is_capped() {
        let td = tempfile::tempdir().unwrap();
        let r = run_with_timeout(
            "head -c 100000 /dev/zero | tr '\\0' 'x'",
            td.path(),
            "",
            Duration::from_secs(10),
        )
        .unwrap();
        assert!(r.truncated);
        assert_eq!(r.stdout.len(), MAX_OUTPUT_BYTES);
    }
}
