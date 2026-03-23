use std::io::{self, Read, Write};
use std::process::{Child, Command, Stdio};

use super::buffer::Buffer;
use super::tube::Tube;
use crate::context;
use crate::error::{Error, Result};

/// Local process tube — the equivalent of `process("./vuln")`.
pub struct Process {
    child: Child,
    stdin: Option<std::process::ChildStdin>,
    stdout: Option<std::process::ChildStdout>,
    buffer: Buffer,
    newline: Vec<u8>,
    path: String,
}

impl Process {
    pub fn new(path: &str, args: &[&str]) -> Result<Self> {
        let mut cmd = Command::new(path);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        let mut child = cmd
            .spawn()
            .map_err(|e| Error::other(format!("failed to spawn {path}: {e}")))?;

        let stdin = child.stdin.take();
        let stdout = child.stdout.take();

        eprintln!(
            "[*] starting process '{path}' (pid {pid})",
            pid = child.id()
        );

        Ok(Self {
            child,
            stdin,
            stdout,
            buffer: Buffer::new(),
            newline: context::get().newline.clone(),
            path: path.to_owned(),
        })
    }

    /// Process ID.
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    /// Check if the process has exited (non-blocking).
    pub fn poll(&mut self) -> Option<std::process::ExitStatus> {
        self.child.try_wait().ok().flatten()
    }

    /// Wait for the process to exit and return its status.
    pub fn wait(&mut self) -> Result<std::process::ExitStatus> {
        self.child.wait().map_err(Error::Io)
    }

    /// Send a signal to the process (Unix only).
    #[cfg(unix)]
    pub fn kill(&mut self) -> Result<()> {
        self.child.kill().map_err(Error::Io)
    }
}

impl Tube for Process {
    fn recv_raw(&mut self, buf: &mut [u8]) -> Result<usize> {
        let stdout = self
            .stdout
            .as_mut()
            .ok_or_else(|| Error::other("stdout unavailable (taken by interactive?)"))?;
        match stdout.read(buf) {
            Ok(n) => Ok(n),
            Err(e) => Err(Error::Io(e)),
        }
    }

    fn send_raw(&mut self, data: &[u8]) -> Result<()> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| Error::other("stdin unavailable"))?;
        stdin.write_all(data)?;
        stdin.flush()?;
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        // Drop stdin so the child sees EOF.
        self.stdin.take();
        let _ = self.child.wait();
        eprintln!("[*] process '{}' stopped", self.path);
        Ok(())
    }

    fn interactive(&mut self) -> Result<()> {
        use std::sync::mpsc;

        eprintln!("[*] switching to interactive mode");

        // Flush pending buffer.
        let pending = self.buffer.get_all();
        if !pending.is_empty() {
            io::stdout().write_all(&pending)?;
            io::stdout().flush()?;
        }

        // Move stdout to reader thread.
        let mut child_stdout = self
            .stdout
            .take()
            .ok_or_else(|| Error::other("stdout unavailable"))?;

        let (done_tx, done_rx) = mpsc::channel::<()>();

        let handle = std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match child_stdout.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let _ = io::stdout().write_all(&buf[..n]);
                        let _ = io::stdout().flush();
                    }
                }
            }
            let _ = done_tx.send(());
        });

        // Main thread: stdin → process stdin.
        let stdin = io::stdin();
        let mut buf = [0u8; 4096];
        loop {
            if done_rx.try_recv().is_ok() {
                break;
            }
            match stdin.lock().read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if self.send_raw(&buf[..n]).is_err() {
                        break;
                    }
                }
            }
        }

        handle.join().ok();
        eprintln!("[*] got EOF — leaving interactive");
        Ok(())
    }

    fn buffer(&self) -> &Buffer {
        &self.buffer
    }
    fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }
    fn newline(&self) -> &[u8] {
        &self.newline
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        self.stdin.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
