use crate::context::{self, Endian};

// ---------------------------------------------------------------------------
// Pack (integer → bytes)
// ---------------------------------------------------------------------------

pub fn p8(val: u8) -> [u8; 1] {
    [val]
}

pub fn p16(val: u16) -> [u8; 2] {
    match context::get().endian {
        Endian::Little => val.to_le_bytes(),
        Endian::Big => val.to_be_bytes(),
    }
}

pub fn p32(val: u32) -> [u8; 4] {
    match context::get().endian {
        Endian::Little => val.to_le_bytes(),
        Endian::Big => val.to_be_bytes(),
    }
}

pub fn p64(val: u64) -> [u8; 8] {
    match context::get().endian {
        Endian::Little => val.to_le_bytes(),
        Endian::Big => val.to_be_bytes(),
    }
}

// ---------------------------------------------------------------------------
// Unpack (bytes → integer)
// Trailing underscore avoids shadowing primitive type names.
// ---------------------------------------------------------------------------

pub fn u8_(data: &[u8]) -> u8 {
    data[0]
}

pub fn u16_(data: &[u8]) -> u16 {
    let buf: [u8; 2] = pad_to(data);
    match context::get().endian {
        Endian::Little => u16::from_le_bytes(buf),
        Endian::Big => u16::from_be_bytes(buf),
    }
}

pub fn u32_(data: &[u8]) -> u32 {
    let buf: [u8; 4] = pad_to(data);
    match context::get().endian {
        Endian::Little => u32::from_le_bytes(buf),
        Endian::Big => u32::from_be_bytes(buf),
    }
}

pub fn u64_(data: &[u8]) -> u64 {
    let buf: [u8; 8] = pad_to(data);
    match context::get().endian {
        Endian::Little => u64::from_le_bytes(buf),
        Endian::Big => u64::from_be_bytes(buf),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Zero-pad (or truncate) `data` into a fixed-size array.
/// Short input is padded with zeroes on the high side (works for both LE/BE
/// since the caller interprets the result).
fn pad_to<const N: usize>(data: &[u8]) -> [u8; N] {
    let mut buf = [0u8; N];
    let len = data.len().min(N);
    buf[..len].copy_from_slice(&data[..len]);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{self, Arch};

    #[test]
    fn roundtrip_le() {
        context::set_arch(Arch::Amd64);
        assert_eq!(u64_(&p64(0xdeadbeef_cafebabe)), 0xdeadbeef_cafebabe);
        assert_eq!(u32_(&p32(0x41414141)), 0x41414141);
    }

    #[test]
    fn roundtrip_be() {
        context::set_arch(Arch::Mips);
        assert_eq!(u32_(&p32(0x41424344)), 0x41424344);
    }

    #[test]
    fn short_unpack() {
        context::set_arch(Arch::Amd64);
        // 6-byte leak, zero-padded
        let leak = u64_(&[0x78, 0x56, 0x34, 0x12, 0xab, 0xcd]);
        assert_eq!(leak, 0x0000_cdab_1234_5678);
    }
}
