use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::Read;
use std::path::Path;
use std::sync::{mpsc::channel, Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct SessionRunner {
    pub pty_system: NativePtySystem,
}

impl SessionRunner {
    pub fn new() -> Self {
        Self {
            pty_system: NativePtySystem::default(),
        }
    }

    pub fn run_command(
        &self,
        cmd: &str,
        args: &[&str],
        cwd: &Path,
        timeout_secs: u64,
    ) -> anyhow::Result<(String, i32)> {
        let pair = self.pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd_builder = CommandBuilder::new(cmd);
        cmd_builder.args(args);
        cmd_builder.cwd(cwd);

        let child = pair.slave.spawn_command(cmd_builder)?;
        let mut reader = pair.master.try_clone_reader()?;

        // Drop slave to close the handle in the parent process.
        // The child has its own copy.
        drop(pair.slave);

        let (output_tx, output_rx) = channel();
        let (status_tx, status_rx) = channel();

        let child = Arc::new(Mutex::new(child));

        // Spawn thread to read output
        let reader_thread = thread::spawn(move || {
            let mut buf = [0u8; 1024];
            let mut output = Vec::new();
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 {
                    break;
                }
                output.extend_from_slice(&buf[..n]);
            }
            let _ = output_tx.send(output);
        });

        // Spawn thread to wait for child process
        let child_clone = Arc::clone(&child);
        let wait_thread = thread::spawn(move || {
            let mut child_guard = child_clone.lock().unwrap();
            match child_guard.wait() {
                Ok(status) => {
                    let _ = status_tx.send(Ok(status));
                }
                Err(e) => {
                    let _ = status_tx.send(Err(e));
                }
            }
        });

        // Wait for status with timeout
        let timeout_duration = Duration::from_secs(timeout_secs);
        let wait_result = status_rx.recv_timeout(timeout_duration);

        let exit_status = match wait_result {
            Ok(Ok(status)) => status,
            Ok(Err(_)) => {
                return Err(anyhow::anyhow!("Failed to wait for child process"));
            }
            Err(_) => {
                // Timeout occurred
                return Err(anyhow::anyhow!(
                    "Command timed out after {} seconds",
                    timeout_secs
                ));
            }
        };

        // Get output (should be ready by now)
        let output = match output_rx.recv() {
            Ok(output) => output,
            Err(_) => Vec::new(),
        };

        let _ = reader_thread.join();
        let _ = wait_thread.join();

        let exit_code = exit_status.exit_code() as i32;
        Ok((String::from_utf8_lossy(&output).to_string(), exit_code))
    }
}
