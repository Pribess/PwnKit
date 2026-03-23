const MR: u8 = 0x80;
const B: u8 = 1;
const W: u8 = 2;
const D: u8 = 3;
const S: u8 = 4;
const XX: u8 = 0xFF;

#[rustfmt::skip]
static OPCODE1: [u8; 256] = [
//  x0      x1      x2      x3      x4      x5      x6      x7      x8      x9      xA      xB      xC      xD      xE      xF
    MR,     MR,     MR,     MR,     B,      D,      0,      0,      MR,     MR,     MR,     MR,     B,      D,      0,      0,      // 0x
    MR,     MR,     MR,     MR,     B,      D,      0,      0,      MR,     MR,     MR,     MR,     B,      D,      0,      0,      // 1x
    MR,     MR,     MR,     MR,     B,      D,      0,      0,      MR,     MR,     MR,     MR,     B,      D,      0,      0,      // 2x
    MR,     MR,     MR,     MR,     B,      D,      0,      0,      MR,     MR,     MR,     MR,     B,      D,      0,      0,      // 3x
    0,      0,      0,      0,      0,      0,      0,      0,      0,      0,      0,      0,      0,      0,      0,      0,      // 4x
    0,      0,      0,      0,      0,      0,      0,      0,      0,      0,      0,      0,      0,      0,      0,      0,      // 5x
    0,      0,      MR,     MR,     0,      0,      0,      0,      D,      MR|D,   B,      MR|B,   0,      0,      0,      0,      // 6x
    B,      B,      B,      B,      B,      B,      B,      B,      B,      B,      B,      B,      B,      B,      B,      B,      // 7x
    MR|B,   MR|D,   MR|B,   MR|B,   MR,     MR,     MR,     MR,     MR,     MR,     MR,     MR,     MR,     MR,     MR,     MR,     // 8x
    0,      0,      0,      0,      0,      0,      0,      0,      0,      0,      XX,     0,      0,      0,      0,      0,      // 9x
    S,      S,      S,      S,      0,      0,      0,      0,      B,      D,      0,      0,      0,      0,      0,      0,      // Ax
    B,      B,      B,      B,      B,      B,      B,      B,      S,      S,      S,      S,      S,      S,      S,      S,      // Bx
    MR|B,   MR|B,   W,      0,      XX,     XX,     MR|B,   MR|D,   S,      0,      W,      0,      0,      B,      0,      0,      // Cx
    MR,     MR,     MR,     MR,     B,      B,      0,      0,      MR,     MR,     MR,     MR,     MR,     MR,     MR,     MR,     // Dx
    B,      B,      B,      B,      B,      B,      B,      B,      D,      D,      XX,     B,      0,      0,      0,      0,      // Ex
    0,      0,      0,      0,      0,      0,      MR|S,   MR|S,   0,      0,      0,      0,      0,      0,      MR,     MR,     // Fx
];

static OPCODE2: [u8; 256] = build_opcode2();

const fn build_opcode2() -> [u8; 256] {
    let mut t = [MR; 256];
    t[0x05] = 0;
    t[0x06] = 0;
    t[0x07] = 0;
    t[0x08] = 0;
    t[0x09] = 0;
    t[0x0B] = 0;
    t[0x0E] = 0;
    t[0x0F] = MR | B;
    t[0x30] = 0;
    t[0x31] = 0;
    t[0x32] = 0;
    t[0x33] = 0;
    t[0x34] = 0;
    t[0x35] = 0;
    t[0x38] = XX;
    t[0x3A] = XX;
    t[0x70] = MR | B;
    t[0x71] = MR | B;
    t[0x72] = MR | B;
    t[0x73] = MR | B;
    t[0x77] = 0;
    let mut i = 0x80;
    while i <= 0x8F {
        t[i] = D;
        i += 1;
    }
    t[0xA0] = 0;
    t[0xA1] = 0;
    t[0xA2] = 0;
    t[0xA4] = MR | B;
    t[0xA8] = 0;
    t[0xA9] = 0;
    t[0xAA] = 0;
    t[0xAC] = MR | B;
    t[0xBA] = MR | B;
    t[0xC2] = MR | B;
    t[0xC4] = MR | B;
    t[0xC5] = MR | B;
    t[0xC6] = MR | B;
    t[0xC8] = 0;
    t[0xC9] = 0;
    t[0xCA] = 0;
    t[0xCB] = 0;
    t[0xCC] = 0;
    t[0xCD] = 0;
    t[0xCE] = 0;
    t[0xCF] = 0;
    t
}

static REGS64: [&str; 16] = [
    "rax", "rcx", "rdx", "rbx", "rsp", "rbp", "rsi", "rdi", "r8", "r9", "r10", "r11", "r12", "r13",
    "r14", "r15",
];

static REGS32: [&str; 16] = [
    "eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi", "r8d", "r9d", "r10d", "r11d", "r12d",
    "r13d", "r14d", "r15d",
];

static REGS8: [&str; 16] = [
    "al", "cl", "dl", "bl", "spl", "bpl", "sil", "dil", "r8b", "r9b", "r10b", "r11b", "r12b",
    "r13b", "r14b", "r15b",
];

static REGS8_NOREX: [&str; 8] = ["al", "cl", "dl", "bl", "ah", "ch", "dh", "bh"];

fn reg(idx: u8, size: u8, has_rex: bool) -> &'static str {
    let i = idx as usize;
    match size {
        64 => REGS64[i],
        32 => REGS32[i],
        8 if has_rex || i >= 8 => REGS8[i],
        8 => REGS8_NOREX[i],
        _ => REGS64[i],
    }
}

pub fn decode_insn(bytes: &[u8], mode64: bool) -> Option<(usize, String)> {
    if bytes.is_empty() {
        return None;
    }

    let mut pos = 0;
    let mut has_66 = false;
    let mut has_67 = false;

    while pos < bytes.len() {
        match bytes[pos] {
            0x26 | 0x2E | 0x36 | 0x3E | 0x64 | 0x65 | 0xF0 | 0xF2 | 0xF3 => pos += 1,
            0x66 => {
                has_66 = true;
                pos += 1;
            }
            0x67 => {
                has_67 = true;
                pos += 1;
            }
            _ => break,
        }
    }
    if pos >= bytes.len() {
        return None;
    }

    let mut rex: u8 = 0;
    if mode64 && (bytes[pos] & 0xF0) == 0x40 {
        rex = bytes[pos];
        pos += 1;
        if pos >= bytes.len() {
            return None;
        }
    }
    let rex_w = rex & 0x08 != 0;
    let rex_r = (rex & 0x04 != 0) as u8;
    let rex_b = (rex & 0x01 != 0) as u8;

    let opcode = bytes[pos];
    pos += 1;

    let (is_two_byte, op, entry) = if opcode == 0x0F {
        if pos >= bytes.len() {
            return None;
        }
        let op2 = bytes[pos];
        pos += 1;
        (true, op2, OPCODE2[op2 as usize])
    } else {
        (false, opcode, OPCODE1[opcode as usize])
    };

    if entry == XX {
        return None;
    }

    let has_modrm = entry & 0x80 != 0;
    let imm_kind = entry & 0x0F;

    let mut modrm_mod: u8 = 0;
    let mut modrm_reg: u8 = 0;
    let mut modrm_rm: u8 = 0;

    if has_modrm {
        if pos >= bytes.len() {
            return None;
        }
        let modrm = bytes[pos];
        pos += 1;
        modrm_mod = (modrm >> 6) & 3;
        modrm_reg = (modrm >> 3) & 7;
        modrm_rm = modrm & 7;

        if modrm_mod != 3 {
            if modrm_rm == 4 {
                if pos >= bytes.len() {
                    return None;
                }
                let sib = bytes[pos];
                pos += 1;
                let sib_base = sib & 7;
                match modrm_mod {
                    0 => {
                        if sib_base == 5 {
                            if pos + 4 > bytes.len() {
                                return None;
                            }
                            pos += 4;
                        }
                    }
                    1 => {
                        if pos >= bytes.len() {
                            return None;
                        }
                        pos += 1;
                    }
                    2 => {
                        if pos + 4 > bytes.len() {
                            return None;
                        }
                        pos += 4;
                    }
                    _ => {}
                }
            } else {
                match modrm_mod {
                    0 => {
                        if modrm_rm == 5 {
                            if pos + 4 > bytes.len() {
                                return None;
                            }
                            pos += 4;
                        }
                    }
                    1 => {
                        if pos >= bytes.len() {
                            return None;
                        }
                        pos += 1;
                    }
                    2 => {
                        if pos + 4 > bytes.len() {
                            return None;
                        }
                        pos += 4;
                    }
                    _ => {}
                }
            }
        }
    }

    let imm_bytes = if imm_kind == S {
        if is_two_byte {
            return None;
        }
        match op {
            0xA0..=0xA3 => {
                if mode64 {
                    if has_67 {
                        4
                    } else {
                        8
                    }
                } else if has_67 {
                    2
                } else {
                    4
                }
            }
            0xB8..=0xBF => {
                if rex_w {
                    8
                } else if has_66 {
                    2
                } else {
                    4
                }
            }
            0xC8 => 3,
            0xF6 => {
                if modrm_reg <= 1 {
                    1
                } else {
                    0
                }
            }
            0xF7 => {
                if modrm_reg <= 1 {
                    if has_66 {
                        2
                    } else {
                        4
                    }
                } else {
                    0
                }
            }
            _ => return None,
        }
    } else {
        match imm_kind {
            0 => 0,
            B => 1,
            W => 2,
            D => {
                if has_66 {
                    2
                } else {
                    4
                }
            }
            _ => return None,
        }
    };

    if pos + imm_bytes > bytes.len() {
        return None;
    }
    pos += imm_bytes;

    let len = pos;
    let text = format_insn(
        bytes,
        len,
        is_two_byte,
        op,
        rex,
        rex_w,
        rex_r,
        rex_b,
        has_66,
        has_modrm,
        modrm_mod,
        modrm_reg,
        modrm_rm,
        mode64,
    );
    Some((len, text))
}

fn operand_size(_op: u8, rex_w: bool, has_66: bool, is_byte_op: bool) -> u8 {
    if is_byte_op {
        return 8;
    }
    if rex_w {
        64
    } else if has_66 {
        16
    } else {
        32
    }
}

fn is_byte_variant(op: u8) -> bool {
    matches!(
        op,
        0x00 | 0x02
            | 0x08
            | 0x0A
            | 0x10
            | 0x12
            | 0x18
            | 0x1A
            | 0x20
            | 0x22
            | 0x28
            | 0x2A
            | 0x30
            | 0x32
            | 0x38
            | 0x3A
            | 0x84
            | 0x86
            | 0x88
            | 0x8A
    )
}

fn alu_mnemonic(op: u8) -> Option<&'static str> {
    let group = if op < 0x40 {
        (op >> 3) & 7
    } else {
        return match op {
            0x84 | 0x85 => Some("test"),
            0x86 | 0x87 => Some("xchg"),
            0x88 | 0x89 | 0x8A | 0x8B => Some("mov"),
            _ => None,
        };
    };
    Some(match group {
        0 => "add",
        1 => "or",
        2 => "adc",
        3 => "sbb",
        4 => "and",
        5 => "sub",
        6 => "xor",
        7 => "cmp",
        _ => return None,
    })
}

fn rm_is_dest(op: u8) -> bool {
    matches!(
        op,
        0x00 | 0x01
            | 0x08
            | 0x09
            | 0x10
            | 0x11
            | 0x18
            | 0x19
            | 0x20
            | 0x21
            | 0x28
            | 0x29
            | 0x30
            | 0x31
            | 0x38
            | 0x39
            | 0x84
            | 0x85
            | 0x86
            | 0x87
            | 0x88
            | 0x89
    )
}

#[allow(clippy::too_many_arguments)]
fn format_insn(
    bytes: &[u8],
    len: usize,
    is_two_byte: bool,
    op: u8,
    rex: u8,
    rex_w: bool,
    rex_r: u8,
    rex_b: u8,
    has_66: bool,
    has_modrm: bool,
    modrm_mod: u8,
    modrm_reg: u8,
    modrm_rm: u8,
    mode64: bool,
) -> String {
    let has_rex = rex != 0;

    if !is_two_byte {
        match op {
            0xC3 => return "ret".into(),
            0x90 if !has_rex && !has_66 => return "nop".into(),
            0xC9 => return "leave".into(),
            0xCC => return "int3".into(),
            0x98 if rex_w => return "cdqe".into(),
            0x98 if has_66 => return "cbw".into(),
            0x98 => return "cwde".into(),
            0x99 if rex_w => return "cqo".into(),
            0x99 if has_66 => return "cwd".into(),
            0x99 => return "cdq".into(),
            0x9C => return "pushfq".into(),
            0x9D => return "popfq".into(),
            0xF4 => return "hlt".into(),
            0xF8 => return "clc".into(),
            0xF9 => return "stc".into(),
            0xFC => return "cld".into(),
            0xFD => return "std".into(),
            _ => {}
        }

        if (0x50..=0x57).contains(&op) {
            let r = (rex_b << 3) | (op & 7);
            let name = if mode64 {
                REGS64[r as usize]
            } else {
                REGS32[r as usize]
            };
            return format!("push {name}");
        }
        if (0x58..=0x5F).contains(&op) {
            let r = (rex_b << 3) | (op & 7);
            let name = if mode64 {
                REGS64[r as usize]
            } else {
                REGS32[r as usize]
            };
            return format!("pop {name}");
        }

        if op == 0xCD {
            let imm = bytes[len - 1];
            return format!("int 0x{imm:02x}");
        }

        if (0x91..=0x97).contains(&op) {
            let r = (rex_b << 3) | (op & 7);
            let sz = operand_size(op, rex_w, has_66, false);
            let ax = reg(0, sz, has_rex);
            let other = reg(r, sz, has_rex);
            return format!("xchg {ax}, {other}");
        }

        if has_modrm && modrm_mod == 3 {
            let r = (rex_r << 3) | modrm_reg;
            let rm = (rex_b << 3) | modrm_rm;

            if op == 0xFF {
                let name = if mode64 {
                    REGS64[rm as usize]
                } else {
                    REGS32[rm as usize]
                };
                return match modrm_reg {
                    2 => format!("call {name}"),
                    4 => format!("jmp {name}"),
                    _ => hex_bytes(bytes, len),
                };
            }

            if let Some(mne) = alu_mnemonic(op) {
                let byte_op = is_byte_variant(op);
                let sz = operand_size(op, rex_w, has_66, byte_op);
                let rn = reg(r, sz, has_rex);
                let rmn = reg(rm, sz, has_rex);
                return if rm_is_dest(op) {
                    format!("{mne} {rmn}, {rn}")
                } else {
                    format!("{mne} {rn}, {rmn}")
                };
            }
        }

        if op == 0xC2 {
            let imm = u16::from_le_bytes([bytes[len - 2], bytes[len - 1]]);
            return format!("ret 0x{imm:x}");
        }
    } else {
        match op {
            0x05 => return "syscall".into(),
            0x34 => return "sysenter".into(),
            0x0B => return "ud2".into(),
            0x1F => return "nop".into(),
            0x31 => return "rdtsc".into(),
            _ => {}
        }

        if has_modrm && modrm_mod == 3 {
            let r = (rex_r << 3) | modrm_reg;
            let rm = (rex_b << 3) | modrm_rm;

            if (0x40..=0x4F).contains(&op) {
                let sz = operand_size(op, rex_w, has_66, false);
                let rn = reg(r, sz, has_rex);
                let rmn = reg(rm, sz, has_rex);
                let cc = [
                    "o", "no", "b", "ae", "e", "ne", "be", "a", "s", "ns", "p", "np", "l", "ge",
                    "le", "g",
                ];
                return format!("cmov{} {rn}, {rmn}", cc[(op & 0xF) as usize]);
            }

            if (0x90..=0x9F).contains(&op) {
                let sz = operand_size(op, rex_w, has_66, false);
                let rmn = reg(rm, 8, has_rex);
                let _ = sz;
                let cc = [
                    "o", "no", "b", "ae", "e", "ne", "be", "a", "s", "ns", "p", "np", "l", "ge",
                    "le", "g",
                ];
                return format!("set{} {rmn}", cc[(op & 0xF) as usize]);
            }

            if op == 0xAF {
                let sz = operand_size(op, rex_w, has_66, false);
                let rn = reg(r, sz, has_rex);
                let rmn = reg(rm, sz, has_rex);
                return format!("imul {rn}, {rmn}");
            }

            if op == 0xB6 || op == 0xB7 {
                let dsz = operand_size(op, rex_w, has_66, false);
                let ssz: u8 = if op == 0xB6 { 8 } else { 16 };
                let _ = ssz;
                let rn = reg(r, dsz, has_rex);
                let rmn = reg(rm, 8, has_rex);
                let mne = if op == 0xB6 { "movzx" } else { "movzx" };
                return format!("{mne} {rn}, {rmn}");
            }

            if op == 0xBE || op == 0xBF {
                let dsz = operand_size(op, rex_w, has_66, false);
                let rn = reg(r, dsz, has_rex);
                let rmn = reg(rm, 8, has_rex);
                return format!("movsx {rn}, {rmn}");
            }
        }
    }

    hex_bytes(bytes, len)
}

fn hex_bytes(bytes: &[u8], len: usize) -> String {
    let parts: Vec<String> = bytes[..len].iter().map(|b| format!("{b:02x}")).collect();
    format!("db {}", parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_ret() {
        assert_eq!(decode_insn(&[0xC3], true), Some((1, "ret".into())));
    }

    #[test]
    fn decode_nop() {
        assert_eq!(decode_insn(&[0x90], true), Some((1, "nop".into())));
    }

    #[test]
    fn decode_pop_rdi() {
        assert_eq!(decode_insn(&[0x5F], true), Some((1, "pop rdi".into())));
    }

    #[test]
    fn decode_pop_r15() {
        assert_eq!(
            decode_insn(&[0x41, 0x5F], true),
            Some((2, "pop r15".into()))
        );
    }

    #[test]
    fn decode_xor_eax_eax() {
        assert_eq!(
            decode_insn(&[0x31, 0xC0], true),
            Some((2, "xor eax, eax".into()))
        );
    }

    #[test]
    fn decode_xor_rax_rax() {
        assert_eq!(
            decode_insn(&[0x48, 0x31, 0xC0], true),
            Some((3, "xor rax, rax".into()))
        );
    }

    #[test]
    fn decode_syscall() {
        assert_eq!(
            decode_insn(&[0x0F, 0x05], true),
            Some((2, "syscall".into()))
        );
    }

    #[test]
    fn decode_int80() {
        assert_eq!(
            decode_insn(&[0xCD, 0x80], false),
            Some((2, "int 0x80".into()))
        );
    }

    #[test]
    fn decode_leave() {
        assert_eq!(decode_insn(&[0xC9], true), Some((1, "leave".into())));
    }

    #[test]
    fn decode_push_rbp() {
        assert_eq!(decode_insn(&[0x55], true), Some((1, "push rbp".into())));
    }

    #[test]
    fn decode_mov_rdi_rax() {
        assert_eq!(
            decode_insn(&[0x48, 0x89, 0xC7], true),
            Some((3, "mov rdi, rax".into()))
        );
    }

    #[test]
    fn decode_call_rax() {
        assert_eq!(
            decode_insn(&[0xFF, 0xD0], true),
            Some((2, "call rax".into()))
        );
    }

    #[test]
    fn decode_jmp_rsi() {
        assert_eq!(
            decode_insn(&[0xFF, 0xE6], true),
            Some((2, "jmp rsi".into()))
        );
    }

    #[test]
    fn decode_multibyte_nop() {
        let bytes = [0x0F, 0x1F, 0x44, 0x00, 0x00];
        let result = decode_insn(&bytes, true);
        assert!(result.is_some());
        let (len, text) = result.unwrap();
        assert_eq!(len, 5);
        assert_eq!(text, "nop");
    }

    #[test]
    fn decode_32bit_pop() {
        assert_eq!(decode_insn(&[0x5F], false), Some((1, "pop edi".into())));
    }

    #[test]
    fn length_modrm_sib_disp() {
        let bytes = [0x8B, 0x44, 0x24, 0x08];
        let result = decode_insn(&bytes, true);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, 4);
    }

    #[test]
    fn length_rip_relative() {
        let bytes = [0x8B, 0x05, 0x00, 0x00, 0x00, 0x00];
        let result = decode_insn(&bytes, true);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, 6);
    }

    #[test]
    fn length_movabs() {
        let bytes = [0x48, 0xB8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let result = decode_insn(&bytes, true);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, 10);
    }

    #[test]
    fn only_consumes_first_insn() {
        let bytes = [0x5F, 0xC3];
        let result = decode_insn(&bytes, true);
        assert_eq!(result, Some((1, "pop rdi".into())));
    }

    #[test]
    fn empty_returns_none() {
        assert_eq!(decode_insn(&[], true), None);
    }
}
