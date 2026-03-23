use std::collections::HashMap;

use crate::context::{self, Arch, Endian};

pub struct SigreturnFrame {
    data: Vec<u8>,
    offsets: HashMap<&'static str, usize>,
}

impl SigreturnFrame {
    pub fn new() -> Self {
        let ctx = context::get();
        match ctx.arch {
            Arch::Amd64 => {
                let mut frame = Self {
                    data: vec![0u8; 248],
                    offsets: HashMap::from([
                        ("uc_flags", 0),
                        ("uc_link", 8),
                        ("ss_sp", 16),
                        ("ss_flags", 24),
                        ("ss_size", 32),
                        ("r8", 40),
                        ("r9", 48),
                        ("r10", 56),
                        ("r11", 64),
                        ("r12", 72),
                        ("r13", 80),
                        ("r14", 88),
                        ("r15", 96),
                        ("rdi", 104),
                        ("rsi", 112),
                        ("rbp", 120),
                        ("rbx", 128),
                        ("rdx", 136),
                        ("rax", 144),
                        ("rcx", 152),
                        ("rsp", 160),
                        ("rip", 168),
                        ("eflags", 176),
                        ("cs", 184),
                        ("gs", 192),
                        ("fs", 200),
                        ("ss", 216),
                    ]),
                };
                frame.set("cs", 0x33, ctx.endian);
                frame.set("ss", 0x2b, ctx.endian);
                frame
            }
            arch => panic!("unsupported sigreturn frame architecture: {arch:?}"),
        }
    }

    pub fn set_rax(&mut self, value: u64) {
        self.set_reg("rax", value);
    }
    pub fn set_rbx(&mut self, value: u64) {
        self.set_reg("rbx", value);
    }
    pub fn set_rcx(&mut self, value: u64) {
        self.set_reg("rcx", value);
    }
    pub fn set_rdx(&mut self, value: u64) {
        self.set_reg("rdx", value);
    }
    pub fn set_rdi(&mut self, value: u64) {
        self.set_reg("rdi", value);
    }
    pub fn set_rsi(&mut self, value: u64) {
        self.set_reg("rsi", value);
    }
    pub fn set_rbp(&mut self, value: u64) {
        self.set_reg("rbp", value);
    }
    pub fn set_rsp(&mut self, value: u64) {
        self.set_reg("rsp", value);
    }
    pub fn set_rip(&mut self, value: u64) {
        self.set_reg("rip", value);
    }
    pub fn set_r8(&mut self, value: u64) {
        self.set_reg("r8", value);
    }
    pub fn set_r9(&mut self, value: u64) {
        self.set_reg("r9", value);
    }
    pub fn set_r10(&mut self, value: u64) {
        self.set_reg("r10", value);
    }
    pub fn set_r11(&mut self, value: u64) {
        self.set_reg("r11", value);
    }
    pub fn set_r12(&mut self, value: u64) {
        self.set_reg("r12", value);
    }
    pub fn set_r13(&mut self, value: u64) {
        self.set_reg("r13", value);
    }
    pub fn set_r14(&mut self, value: u64) {
        self.set_reg("r14", value);
    }
    pub fn set_r15(&mut self, value: u64) {
        self.set_reg("r15", value);
    }
    pub fn set_eflags(&mut self, value: u64) {
        self.set_reg("eflags", value);
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    fn set_reg(&mut self, name: &'static str, value: u64) {
        self.set(name, value, context::get().endian);
    }

    fn set(&mut self, name: &'static str, value: u64, endian: Endian) {
        let offset = self.offsets[name];
        let bytes = match endian {
            Endian::Little => value.to_le_bytes(),
            Endian::Big => value.to_be_bytes(),
        };
        self.data[offset..offset + 8].copy_from_slice(&bytes);
    }
}
