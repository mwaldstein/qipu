use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::Read;
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;

pub struct SessionRunner {
    pub pty_system: NativePtySystem,
}

impl SessionRunner {
    pub fn new() -> Self {
        Self {
            pty_system: NativePtySystem::default(),
        }
    }

    pub fn run_command(&self, cmd: &str, args: &[&str], cwd: &Path) -> anyhow::Result<String> {
        let pair = self.pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd_builder = CommandBuilder::new(cmd);
        cmd_builder.args(args);
        cmd_builder.cwd(cwd);

        let mut child = pair.slave.spawn_command(cmd_builder)?;
        let mut reader = pair.master.try_clone_reader()?;

        // Drop slave to close the handle in the parent process.
        // The child has its own copy.
        drop(pair.slave);

        let (tx, rx) = channel();

        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            let mut output = Vec::new();
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 {
                    break;
                }
                output.extend_from_slice(&buf[..n]);
            }
            let _ = tx.send(output);
        });

        child.wait()?;
        let output = rx.recv()?;

        Ok(String::from_utf8_lossy(&output).to_string())
    }
}
