use super::buffer::Buffer;
use crate::error::{Error, Result};

/// Core I/O abstraction — every transport (TCP, process, SSH, …)
/// implements this trait and gets the full recv/send API for free.
pub trait Tube {
    // ----- required (transport-specific) --------------------------------

    /// Low-level read into `buf`. Returns number of bytes read.
    /// Must return `Err(Error::Timeout)` on timeout, `Ok(0)` on EOF.
    fn recv_raw(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// Low-level write.
    fn send_raw(&mut self, data: &[u8]) -> Result<()>;

    /// Shut down the transport.
    fn close(&mut self) -> Result<()>;

    /// Hand control to the user (stdin ↔ tube).
    fn interactive(&mut self) -> Result<()>;

    /// Access the internal buffer.
    fn buffer(&self) -> &Buffer;
    fn buffer_mut(&mut self) -> &mut Buffer;

    /// Line terminator used by `sendline` / `recvline`.
    fn newline(&self) -> &[u8];

    // ----- provided (convenience) ---------------------------------------

    /// Receive up to `n` bytes (returns immediately once *any* data arrives).
    fn recv(&mut self, n: usize) -> Result<Vec<u8>> {
        let buf_len = self.buffer().len();
        if buf_len > 0 {
            let take = n.min(buf_len);
            return Ok(self.buffer_mut().get(take));
        }
        let mut tmp = vec![0u8; n];
        let read = self.recv_raw(&mut tmp)?;
        if read == 0 {
            return Err(Error::ConnectionClosed);
        }
        Ok(tmp[..read].to_vec())
    }

    /// Receive exactly `n` bytes (blocks until all received).
    fn recvn(&mut self, n: usize) -> Result<Vec<u8>> {
        let mut out = Vec::with_capacity(n);
        while out.len() < n {
            let remaining = n - out.len();
            let blen = self.buffer().len();
            if blen > 0 {
                let take = remaining.min(blen);
                out.extend_from_slice(&self.buffer_mut().get(take));
                continue;
            }
            let mut tmp = vec![0u8; remaining];
            let read = self.recv_raw(&mut tmp)?;
            if read == 0 {
                return Err(Error::ConnectionClosed);
            }
            out.extend_from_slice(&tmp[..read]);
        }
        Ok(out)
    }

    /// Receive until `delim` is found.
    /// If `drop` is true the delimiter is consumed but not returned.
    fn recv_until(&mut self, delim: &[u8], drop: bool) -> Result<Vec<u8>> {
        loop {
            if let Some(hit) = self.buffer_mut().get_until(delim, drop) {
                return Ok(hit);
            }
            let mut tmp = vec![0u8; 4096];
            let read = self.recv_raw(&mut tmp)?;
            if read == 0 {
                return Err(Error::ConnectionClosed);
            }
            self.buffer_mut().add(&tmp[..read]);
        }
    }

    /// Receive a single line (strips the trailing newline).
    fn recvline(&mut self) -> Result<Vec<u8>> {
        let nl = self.newline().to_vec();
        self.recv_until(&nl, true)
    }

    /// Send raw bytes.
    fn send(&mut self, data: &[u8]) -> Result<()> {
        self.send_raw(data)
    }

    /// Send bytes followed by a newline.
    fn sendline(&mut self, data: &[u8]) -> Result<()> {
        let nl = self.newline().to_vec();
        let mut payload = data.to_vec();
        payload.extend_from_slice(&nl);
        self.send_raw(&payload)
    }

    /// Receive until `delim`, then send `data`.
    fn sendafter(&mut self, delim: &[u8], data: &[u8]) -> Result<()> {
        self.recv_until(delim, true)?;
        self.send(data)
    }

    /// Receive until `delim`, then sendline `data`.
    fn sendlineafter(&mut self, delim: &[u8], data: &[u8]) -> Result<()> {
        self.recv_until(delim, true)?;
        self.sendline(data)
    }
}
