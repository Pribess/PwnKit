//! **PwnKit** — CTF exploit development toolkit, rewritten in Rust.
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
