use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

use super::buffer::Buffer;
use super::tube::Tube;
use crate::context;
use crate::error::{Error, Result};

/// TCP connection tube — the equivalent of `remote("host", port)`.
pub struct Remote {
    stream: TcpStream,
    buffer: Buffer,
    newline: Vec<u8>,
    host: String,
    port: u16,
}

impl Remote {
    pub fn new(host: &str, port: u16) -> Result<Self> {
        let addr = (host, port)
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| Error::other(format!("cannot resolve {host}:{port}")))?;

        let stream = TcpStream::connect(addr)?;
        let timeout = context::get().timeout;
        stream.set_read_timeout(timeout)?;
        stream.set_write_timeout(timeout)?;

        eprintln!("[*] opening connection to {host}:{port}");

        Ok(Self {
            stream,
            buffer: Buffer::new(),
            newline: context::get().newline.clone(),
            host: host.to_owned(),
            port,
        })
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Tube for Remote {
    fn recv_raw(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.stream.read(buf) {
            Ok(n) => Ok(n),
            Err(e)
                if e.kind() == io::ErrorKind::TimedOut || e.kind() == io::ErrorKind::WouldBlock =>
            {
                Err(Error::Timeout)
            }
            Err(e) => Err(Error::Io(e)),
        }
    }

    fn send_raw(&mut self, data: &[u8]) -> Result<()> {
        self.stream.write_all(data)?;
        self.stream.flush()?;
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        eprintln!("[*] closing connection to {}:{}", self.host, self.port);
        self.stream.shutdown(std::net::Shutdown::Both)?;
        Ok(())
    }

    fn interactive(&mut self) -> Result<()> {
        use std::sync::mpsc;

        eprintln!("[*] switching to interactive mode");

        // Flush anything still in the buffer.
        let pending = self.buffer.get_all();
        if !pending.is_empty() {
            io::stdout().write_all(&pending)?;
            io::stdout().flush()?;
        }

        // Clone the TCP stream so the reader thread has its own handle.
        let mut reader = self.stream.try_clone()?;
        // Remove read timeout for interactive — we want blocking reads.
        reader.set_read_timeout(None)?;

        let (done_tx, done_rx) = mpsc::channel::<()>();

        // Reader thread: tube → stdout.
        let handle = std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let _ = io::stdout().write_all(&buf[..n]);
                        let _ = io::stdout().flush();
                    }
                }
            }
            let _ = done_tx.send(());
        });

        // Main thread: stdin → tube.
        let stdin = io::stdin();
        let mut buf = [0u8; 4096];
        loop {
            if done_rx.try_recv().is_ok() {
                break;
            }
            match stdin.lock().read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if self.stream.write_all(&buf[..n]).is_err() {
                        break;
                    }
                    let _ = self.stream.flush();
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

impl Drop for Remote {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }
}
