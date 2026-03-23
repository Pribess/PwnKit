use std::collections::HashSet;

use crate::elf::ELF;

pub struct Gadget {
    pub address: u64,
    pub insns: String,
}

pub fn find_gadgets(elf: &ELF, depth: usize) -> Vec<Gadget> {
    let mut seen = HashSet::new();
    let mut gadgets = Vec::new();

    for (segment_addr, segment) in elf.executable_segments() {
        for (ret_offset, byte) in segment.iter().enumerate() {
            if *byte != 0xc3 {
                continue;
            }

            let start = ret_offset.saturating_sub(depth);
            for gadget_start in start..=ret_offset {
                let slice = &segment[gadget_start..=ret_offset];
                let address = *segment_addr + gadget_start as u64;
                if seen.contains(&address) {
                    continue;
                }
                if let Some(insns) = decode_gadget(slice, elf.bits(), address) {
                    seen.insert(address);
                    gadgets.push(Gadget { address, insns });
                }
            }
        }
    }

    gadgets.sort_by_key(|gadget| gadget.address);
    gadgets
}

pub fn find_gadget(elf: &ELF, pattern: &[&str]) -> Option<u64> {
    let patterns: Vec<String> = pattern
        .iter()
        .map(|part| part.to_ascii_lowercase())
        .collect();
    find_gadgets(elf, 20)
        .into_iter()
        .find(|gadget| {
            let insns: Vec<String> = gadget
                .insns
                .split("; ")
                .map(|insn| insn.to_ascii_lowercase())
                .collect();
            insns.len() == patterns.len()
                && insns
                    .iter()
                    .zip(patterns.iter())
                    .all(|(insn, pat)| insn.contains(pat))
        })
        .map(|gadget| gadget.address)
}

fn decode_gadget(bytes: &[u8], bits: u32, _ip: u64) -> Option<String> {
    let mode64 = bits == 64;
    let mut pos = 0;
    let mut parts = Vec::new();

    while pos < bytes.len() {
        let (len, text) = super::decode::decode_insn(&bytes[pos..], mode64)?;
        if len == 0 {
            return None;
        }
        parts.push(text);
        pos += len;
    }

    if pos != bytes.len() {
        return None;
    }
    if !matches!(parts.last(), Some(last) if last.eq_ignore_ascii_case("ret")) {
        return None;
    }

    Some(parts.join("; "))
}
