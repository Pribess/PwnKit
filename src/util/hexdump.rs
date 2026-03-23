use std::fmt::Write;

/// Produce a classic hexdump of `data`.
///
/// ```
/// use pwnkit::hexdump;
///
/// let s = hexdump(b"Hello, pwnkit!\n");
/// assert!(s.contains("48 65 6c 6c"));
/// ```
pub fn hexdump(data: &[u8]) -> String {
    let mut out = String::new();

    for (i, chunk) in data.chunks(16).enumerate() {
        // Offset
        let _ = write!(out, "{:08x}  ", i * 16);

        // Hex bytes
        for (j, byte) in chunk.iter().enumerate() {
            if j == 8 {
                out.push(' ');
            }
            let _ = write!(out, "{byte:02x} ");
        }

        // Padding for short last line
        let missing = 16 - chunk.len();
        for _ in 0..missing {
            out.push_str("   ");
        }
        if chunk.len() <= 8 {
            out.push(' ');
        }

        // ASCII
        out.push_str(" |");
        for &b in chunk {
            if b.is_ascii_graphic() || b == b' ' {
                out.push(b as char);
            } else {
                out.push('.');
            }
        }
        out.push_str("|\n");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let dump = hexdump(b"AAAA\x00\x01\x02\x03");
        assert!(dump.contains("41 41 41 41 00 01 02 03"));
        assert!(dump.contains("|AAAA....|"));
    }
}
