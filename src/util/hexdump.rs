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
    fn empty_data() {
        assert_eq!(hexdump(b""), "");
    }

    #[test]
    fn single_byte() {
        let dump = hexdump(&[0x41]);
        assert!(dump.contains("00000000"));
        assert!(dump.contains("41"));
        assert!(dump.contains("|A|"));
    }

    #[test]
    fn full_16_bytes() {
        let data: Vec<u8> = (0x40..0x50).collect();
        let dump = hexdump(&data);
        assert!(dump.contains("40 41 42 43 44 45 46 47"));
        assert!(dump.contains("48 49 4a 4b 4c 4d 4e 4f"));
    }

    #[test]
    fn nonprintable_shows_dot() {
        let dump = hexdump(&[0x00, 0x01, 0x02, 0x7f]);
        assert!(dump.contains("|....|"));
    }

    #[test]
    fn multiline() {
        let data = vec![0x41; 32];
        let dump = hexdump(&data);
        assert!(dump.contains("00000000"));
        assert!(dump.contains("00000010"));
    }

    #[test]
    fn smoke() {
        let dump = hexdump(b"AAAA\x00\x01\x02\x03");
        assert!(dump.contains("41 41 41 41 00 01 02 03"));
        assert!(dump.contains("|AAAA....|"));
    }
}
