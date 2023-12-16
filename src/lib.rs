#![no_std]
#![feature(core_intrinsics)]
#![feature(naked_functions)]

// Syscalls.
pub mod caps;
pub mod stats;
pub mod syscalls;
pub use syscalls::SysHandle;

#[cfg(feature = "userspace")]
pub mod time;

#[cfg(not(feature = "rustc-dep-of-std"))]
extern crate alloc;
use alloc::string::String;

// Kernel/usperspace shared memory.
mod shared_mem;
pub use shared_mem::*;

#[cfg(feature = "userspace")]
pub fn align_up(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two(), "`align` must be a power of two");
    let align_mask = align - 1;
    if addr & align_mask == 0 {
        addr // already aligned
    } else {
        (addr | align_mask) + 1
    }
}

// #[cfg(not(feature = "rustc-dep-of-std"))]
pub fn url_encode(url: &str) -> String {
    // Replace ':' with '&col'; '=' with '&eq'; '&' with '&amp;'.
    let amps = url.replace('&', "&amp;");
    let cols = amps.replace(':', "&col;");

    cols.replace('=', "&eq;")
}

#[cfg(not(feature = "rustc-dep-of-std"))]
pub fn url_decode(encoded: &str) -> String {
    let eqs = encoded.replace("&eq;", "=");
    let cols = eqs.replace("&col;", ":");

    cols.replace("&amp;", "&")
}

/*
#[cfg(feature = "userspace")]
pub fn utid() -> u64 {
    let mut fsbase: u64;
    unsafe {
        core::arch::asm!("rdfsbase {}", out(reg) fsbase, options(nostack, preserves_flags));
    }

    fsbase
}
*/

#[cfg(feature = "userspace")]
pub fn __utid() -> u64 {
    shared_mem::UserThreadControlBlock::__utid()
}

#[cfg(feature = "userspace")]
pub fn current_cpu() -> u32 {
    shared_mem::UserThreadControlBlock::get().current_cpu
}

#[cfg(feature = "userspace")]
pub fn num_cpus() -> u32 {
    KernelStaticPage::get().num_cpus
}

// Most system-level APIs (syscalls, IO drivers) return 16-bit error codes
// to make things simple (errno works well enough in Linux/POSIX).
// Applications that want to use more sophisticated errors are free to do that.
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    Ok = 0,
    UnspecifiedError = 1, // A generic error.
    UnknownError = 2,     // Should only be used in from_u16() below.
    NotReady = 3,
    NotImplemented = 5,
    VersionTooHigh = 6,
    VersionTooLow = 7,
    InvalidArgument = 8,
    OutOfMemory = 9,
    NotAllowed = 10, // Permission error.
    NotFound = 11,
    InternalError = 12,
    TimedOut = 13,
    AlreadyInUse = 14,
    UnexpectedEof = 15,
    InvalidFilename = 16,
    NotADirectory = 17,
    BadHandle = 18,
    FileTooLarge = 19,

    MaxKernelError, // Must be last, so that from_u16() below works.
}

impl ErrorCode {
    pub fn is_ok(&self) -> bool {
        *self == ErrorCode::Ok
    }

    pub fn is_err(&self) -> bool {
        *self != ErrorCode::Ok
    }

    pub fn from_u16(val: u16) -> Self {
        if val >= Self::MaxKernelError as u16 {
            Self::UnknownError
        } else {
            unsafe { core::mem::transmute(val) }
        }
    }
}

impl From<ErrorCode> for u16 {
    fn from(value: ErrorCode) -> Self {
        value as u16
    }
}

impl From<u16> for ErrorCode {
    fn from(value: u16) -> Self {
        Self::from_u16(value)
    }
}
