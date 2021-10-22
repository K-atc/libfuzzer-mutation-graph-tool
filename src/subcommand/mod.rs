#[cfg(feature = "afl")]
pub mod afl;
#[cfg(feature = "libfuzzer")]
pub mod libfuzzer;

pub mod util;
