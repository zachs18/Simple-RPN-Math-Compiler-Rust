#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub(crate) use x86_64::*;

#[cfg(target_arch = "x86")]
mod x86;
#[cfg(target_arch = "x86")]
pub(crate) use x86::*;

#[cfg(target_arch = "arm")]
mod arm;
#[cfg(target_arch = "arm")]
pub(crate) use arm::*;

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "aarch64")]
pub(crate) use aarch64::*;
