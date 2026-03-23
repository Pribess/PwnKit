//! # PwnKit
//!
//! Zero-dependency CTF exploit development toolkit — pwntools, rewritten in Rust.
//!
//! ## Quick Start
//!
//! ```no_run
//! use pwnkit::*;
//! use pwnkit::tubes::Tube;
//!
//! fn main() -> Result<()> {
//!     context::set_arch(Arch::Amd64);
//!
//!     let mut io = remote("localhost", 1337)?;
//!     io.recv_until(b"> ", true)?;
//!     io.sendline(&p64(0xdeadbeef));
//!     io.interactive()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Context
//!
//! Global thread-local context controls architecture, endianness, and OS.
//! Setting the architecture automatically configures word size and byte order.
//!
//! ```
//! use pwnkit::*;
//!
//! context::set_arch(Arch::Amd64);
//! assert_eq!(context::get().bits, 64);
//! assert_eq!(context::get().endian, Endian::Little);
//!
//! context::set_arch(Arch::Ppc);
//! assert_eq!(context::get().bits, 32);
//! assert_eq!(context::get().endian, Endian::Big);
//! ```
//!
//! Scoped overrides with automatic restore:
//!
//! ```
//! use pwnkit::*;
//!
//! context::set_arch(Arch::Amd64);
//! {
//!     let _g = context::local(|ctx| ctx.set_arch(Arch::I386));
//!     assert_eq!(context::get().bits, 32);
//! }
//! assert_eq!(context::get().bits, 64);
//! ```
//!
//! ## Packing
//!
//! Pack and unpack integers with context-aware endianness.
//!
//! ```
//! use pwnkit::*;
//!
//! context::set_arch(Arch::Amd64);
//! assert_eq!(p32(0xdeadbeef), [0xef, 0xbe, 0xad, 0xde]);
//! assert_eq!(u32_(&[0xef, 0xbe, 0xad, 0xde]), 0xdeadbeef);
//!
//! // Big endian
//! context::set_endian(Endian::Big);
//! assert_eq!(p32(0x41424344), *b"ABCD");
//! assert_eq!(p64(0x4142434445464748), *b"ABCDEFGH");
//!
//! // Short unpack (6-byte libc leak, zero-padded)
//! context::set_arch(Arch::Amd64);
//! let leak = u64_(&[0x78, 0x56, 0x34, 0x12, 0xab, 0xcd]);
//! assert_eq!(leak, 0x0000_cdab_1234_5678);
//! ```
//!
//! ## Tubes
//!
//! Uniform I/O over TCP and local processes.
//!
//! ```no_run
//! use pwnkit::*;
//! use pwnkit::tubes::Tube;
//!
//! // TCP
//! let mut io = remote("pwn.example.com", 1337)?;
//!
//! // Local process
//! let mut io = process("./vuln")?;
//!
//! io.recv_until(b"Name: ", true)?;
//! io.sendline(b"admin");
//! io.sendlineafter(b"Password: ", b"hunter2")?;
//! let line = io.recvline()?;
//!
//! io.interactive()?;
//! # Ok::<(), pwnkit::Error>(())
//! ```
//!
//! ## Cyclic Patterns
//!
//! De Bruijn sequences for finding crash offsets.
//!
//! ```
//! use pwnkit::*;
//!
//! assert_eq!(cyclic(20), b"aaaabaaacaaadaaaeaaa");
//! assert_eq!(cyclic_find(b"baaa"), Some(4));
//!
//! let pattern = cyclic(1000);
//! assert_eq!(cyclic_find(&pattern[514..518]), Some(514));
//! ```
//!
//! ## Payload Building
//!
//! Concatenate heterogeneous byte slices with the [`flat!`] macro.
//!
//! ```
//! use pwnkit::*;
//!
//! context::set_arch(Arch::Amd64);
//! let payload = flat![
//!     b"A".repeat(40),
//!     p64(0x00401234),
//!     p64(0x00007fff_deadbeef),
//! ];
//! assert_eq!(payload.len(), 40 + 8 + 8);
//! ```
//!
//! ## ELF Parsing
//!
//! Parse symbols, GOT, PLT, and search binary data — no external dependencies.
//!
//! ```no_run
//! use pwnkit::*;
//!
//! let elf = ELF::new("./vuln")?;
//! let main = elf.sym("main");
//! let puts_got = elf.got("puts");
//! let puts_plt = elf.plt("puts");
//!
//! // Search for "/bin/sh" in libc
//! let libc = ELF::new("./libc.so.6")?;
//! let binsh = libc.search(b"/bin/sh").unwrap();
//!
//! // ASLR rebasing
//! # let leaked_addr: u64 = 0;
//! let mut libc = ELF::new("./libc.so.6")?;
//! libc.set_base(leaked_addr - libc.sym("puts"));
//! let system = libc.sym("system");
//! # Ok::<(), pwnkit::Error>(())
//! ```
//!
//! ## ROP
//!
//! Build ROP chains and find gadgets with the built-in x86/x64 decoder.
//!
//! ```no_run
//! use pwnkit::*;
//!
//! let elf = ELF::new("./vuln")?;
//!
//! // Find gadgets
//! let pop_rdi = find_gadget(&elf, &["pop rdi", "ret"]).unwrap();
//! let ret = find_gadget(&elf, &["ret"]).unwrap();
//!
//! // Build chain
//! let mut rop = ROP::new(&elf);
//! rop.raw(pop_rdi);
//! rop.raw(0x00402004);  // "/bin/sh"
//! rop.raw(ret);         // stack alignment
//! rop.raw(elf.plt("system"));
//!
//! let payload = flat![
//!     b"A".repeat(72),
//!     rop.chain(),
//! ];
//! # Ok::<(), pwnkit::Error>(())
//! ```
//!
//! ## SROP
//!
//! Sigreturn-oriented programming frames.
//!
//! ```
//! use pwnkit::*;
//!
//! context::set_arch(Arch::Amd64);
//! let mut frame = SigreturnFrame::new();
//! frame.set_rax(59);         // execve
//! frame.set_rdi(0x402000);   // "/bin/sh"
//! frame.set_rsi(0);
//! frame.set_rdx(0);
//! frame.set_rip(0x401050);   // syscall
//!
//! let payload = frame.as_bytes();
//! assert_eq!(payload.len(), 248);
//! ```
//!
//! ## Format String
//!
//! Generate format string payloads for arbitrary writes.
//!
//! ```
//! use pwnkit::*;
//!
//! context::set_arch(Arch::Amd64);
//! let payload = fmtstr_payload(6, &[(0x404000, 0x42)]);
//! let s = String::from_utf8_lossy(&payload);
//! assert!(s.contains("$hhn"));
//! ```
//!
//! ## Hexdump
//!
//! ```
//! use pwnkit::hexdump;
//!
//! let s = hexdump(b"Hello, PwnKit!\x00\x01\x02");
//! assert!(s.contains("48 65 6c 6c"));
//! ```

pub mod context;
pub mod elf;
pub mod error;
pub mod fmt;
pub mod pack;
pub mod rop;
pub mod tubes;
pub mod util;

pub use context::{Arch, Context, Endian, Os};
pub use elf::ELF;
pub use error::{Error, Result};
pub use fmt::fmtstr_payload;
pub use pack::{p16, p32, p64, p8, u16_, u32_, u64_, u8_};
pub use rop::{find_gadget, find_gadgets, Gadget, SigreturnFrame, ROP};
pub use tubes::{Buffer, Process, Remote};
pub use util::{cyclic, cyclic_find, hexdump};

pub fn remote(host: &str, port: u16) -> Result<Remote> {
    Remote::new(host, port)
}

pub fn process(path: &str) -> Result<Process> {
    Process::new(path, &[])
}

pub fn process_with_args(path: &str, args: &[&str]) -> Result<Process> {
    Process::new(path, args)
}
