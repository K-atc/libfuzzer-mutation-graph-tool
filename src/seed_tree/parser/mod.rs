pub mod error;
pub mod result;

pub mod generic;

#[cfg(feature = "afl")]
pub mod afl;
#[cfg(feature = "libfuzzer")]
pub mod libfuzzer;
