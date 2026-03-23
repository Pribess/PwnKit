use std::cell::RefCell;
use std::time::Duration;

/// Target architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    Amd64,
    I386,
    Arm,
    Aarch64,
    Mips,
    Mips64,
    Ppc,
    Ppc64,
    Riscv32,
    Riscv64,
}

impl Arch {
    /// Native word size in bits.
    pub fn bits(self) -> u32 {
        match self {
            Self::Amd64 | Self::Aarch64 | Self::Mips64 | Self::Ppc64 | Self::Riscv64 => 64,
            Self::I386 | Self::Arm | Self::Mips | Self::Ppc | Self::Riscv32 => 32,
        }
    }

    /// Default byte order for this architecture.
    pub fn endian(self) -> Endian {
        match self {
            Self::Ppc | Self::Ppc64 => Endian::Big,
            _ => Endian::Little,
        }
    }

    /// Number of bytes in a native word.
    pub fn bytes(self) -> u32 {
        self.bits() / 8
    }
}

/// Byte order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Little,
    Big,
}

/// Target operating system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Linux,
    FreeBSD,
    Windows,
}

/// Execution context — architecture, endianness, OS, timeout, etc.
#[derive(Debug, Clone)]
pub struct Context {
    pub arch: Arch,
    pub endian: Endian,
    pub bits: u32,
    pub os: Os,
    pub timeout: Option<Duration>,
    pub newline: Vec<u8>,
}

impl Context {
    /// Update the architecture and automatically adjust `bits` and `endian`.
    pub fn set_arch(&mut self, arch: Arch) {
        self.arch = arch;
        self.bits = arch.bits();
        self.endian = arch.endian();
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            arch: Arch::Amd64,
            endian: Endian::Little,
            bits: 64,
            os: Os::Linux,
            timeout: None,
            newline: b"\n".to_vec(),
        }
    }
}

// ---------------------------------------------------------------------------
// Thread-local context stack
// ---------------------------------------------------------------------------

thread_local! {
    static CTX_STACK: RefCell<Vec<Context>> = RefCell::new(vec![Context::default()]);
}

/// Return a snapshot of the current context.
pub fn get() -> Context {
    CTX_STACK.with(|s| s.borrow().last().expect("context stack empty").clone())
}

/// Set the target architecture (also updates `bits` and `endian`).
pub fn set_arch(arch: Arch) {
    update(|ctx| {
        ctx.arch = arch;
        ctx.bits = arch.bits();
        ctx.endian = arch.endian();
    });
}

/// Set the byte order.
pub fn set_endian(endian: Endian) {
    update(|ctx| ctx.endian = endian);
}

/// Set the target OS.
pub fn set_os(os: Os) {
    update(|ctx| ctx.os = os);
}

/// Set the global I/O timeout.
pub fn set_timeout(timeout: Duration) {
    update(|ctx| ctx.timeout = Some(timeout));
}

/// Clear the global I/O timeout (blocking reads).
pub fn clear_timeout() {
    update(|ctx| ctx.timeout = None);
}

/// Mutate the current context in-place.
pub fn update(f: impl FnOnce(&mut Context)) {
    CTX_STACK.with(|s| {
        let mut stack = s.borrow_mut();
        let ctx = stack.last_mut().expect("context stack empty");
        f(ctx);
    });
}

// ---------------------------------------------------------------------------
// Scoped overrides
// ---------------------------------------------------------------------------

/// RAII guard that pops the context stack on drop.
pub struct ContextGuard;

impl Drop for ContextGuard {
    fn drop(&mut self) {
        CTX_STACK.with(|s| {
            let mut stack = s.borrow_mut();
            if stack.len() > 1 {
                stack.pop();
            }
        });
    }
}

/// Push a scoped context override. Returns a guard — the override
/// is reverted when the guard is dropped.
///
/// ```
/// use pwnkit::context::{self, Arch};
///
/// context::set_arch(Arch::Amd64);
///
/// {
///     let _g = context::local(|ctx| ctx.set_arch(Arch::I386));
///     assert_eq!(context::get().bits, 32);
/// }
///
/// assert_eq!(context::get().bits, 64);
/// ```
pub fn local(f: impl FnOnce(&mut Context)) -> ContextGuard {
    CTX_STACK.with(|s| {
        let mut stack = s.borrow_mut();
        let mut fork = stack.last().expect("context stack empty").clone();
        f(&mut fork);
        stack.push(fork);
    });
    ContextGuard
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arch_arm() {
        set_arch(Arch::Arm);
        let c = get();
        assert_eq!(c.arch, Arch::Arm);
        assert_eq!(c.bits, 32);
        assert_eq!(c.endian, Endian::Little);
    }

    #[test]
    fn arch_aarch64() {
        set_arch(Arch::Aarch64);
        let c = get();
        assert_eq!(c.bits, 64);
        assert_eq!(c.endian, Endian::Little);
    }

    #[test]
    fn arch_i386() {
        set_arch(Arch::I386);
        let c = get();
        assert_eq!(c.bits, 32);
        assert_eq!(c.endian, Endian::Little);
    }

    #[test]
    fn arch_amd64() {
        set_arch(Arch::Amd64);
        let c = get();
        assert_eq!(c.bits, 64);
        assert_eq!(c.endian, Endian::Little);
    }

    #[test]
    fn arch_mips() {
        set_arch(Arch::Mips);
        let c = get();
        assert_eq!(c.bits, 32);
        assert_eq!(c.endian, Endian::Little);
    }

    #[test]
    fn arch_mips64() {
        set_arch(Arch::Mips64);
        let c = get();
        assert_eq!(c.bits, 64);
        assert_eq!(c.endian, Endian::Little);
    }

    #[test]
    fn arch_ppc() {
        set_arch(Arch::Ppc);
        let c = get();
        assert_eq!(c.bits, 32);
        assert_eq!(c.endian, Endian::Big);
    }

    #[test]
    fn arch_ppc64() {
        set_arch(Arch::Ppc64);
        let c = get();
        assert_eq!(c.bits, 64);
        assert_eq!(c.endian, Endian::Big);
    }

    #[test]
    fn scoped_local() {
        set_arch(Arch::Arm);
        {
            let _g = local(|ctx| ctx.set_arch(Arch::I386));
            assert_eq!(get().arch, Arch::I386);
            assert_eq!(get().bits, 32);
        }
        assert_eq!(get().arch, Arch::Arm);
    }
}
