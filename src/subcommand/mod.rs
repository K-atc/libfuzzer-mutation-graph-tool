#[cfg(feature = "afl")]
pub(crate) mod afl;
#[cfg(feature = "libfuzzer")]
pub(crate) mod libfuzzer;

pub(crate) mod common;
pub(crate) mod util;
