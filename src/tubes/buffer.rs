/// Byte buffer used internally by every tube to accumulate received data.
#[derive(Debug)]
pub struct Buffer {
    data: Vec<u8>,
}

impl Buffer {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Append raw bytes to the buffer.
    pub fn add(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    /// Drain and return up to `n` bytes from the front.
    pub fn get(&mut self, n: usize) -> Vec<u8> {
        let n = n.min(self.data.len());
        let out = self.data[..n].to_vec();
        self.data.drain(..n);
        out
    }

    /// Search for `needle` in the buffered data.
    pub fn find(&self, needle: &[u8]) -> Option<usize> {
        if needle.is_empty() || needle.len() > self.data.len() {
            return None;
        }
        self.data.windows(needle.len()).position(|w| w == needle)
    }

    /// If `delim` is present, drain everything up to (and including) the
    /// delimiter.  When `drop` is true the delimiter itself is excluded from
    /// the returned data.
    pub fn get_until(&mut self, delim: &[u8], drop: bool) -> Option<Vec<u8>> {
        let pos = self.find(delim)?;
        let end = pos + delim.len();
        let out = if drop {
            self.data[..pos].to_vec()
        } else {
            self.data[..end].to_vec()
        };
        self.data.drain(..end);
        Some(out)
    }

    /// Drain everything currently buffered.
    pub fn get_all(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.data)
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_add_get() {
        let mut b = Buffer::new();
        b.add(b"hello ");
        b.add(b"world");
        assert_eq!(b.len(), 11);
        assert_eq!(b.get(5), b"hello");
        assert_eq!(b.get(100), b" world");
        assert!(b.is_empty());
    }

    #[test]
    fn get_until_drop() {
        let mut b = Buffer::new();
        b.add(b"Enter name: foobar\n");
        let out = b.get_until(b": ", true).unwrap();
        assert_eq!(out, b"Enter name");
        assert_eq!(&b.get_all(), b"foobar\n");
    }

    #[test]
    fn get_until_keep() {
        let mut b = Buffer::new();
        b.add(b"prompt> data");
        let out = b.get_until(b"> ", false).unwrap();
        assert_eq!(out, b"prompt> ");
        assert_eq!(&b.get_all(), b"data");
    }
}
