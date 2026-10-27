//! zenhash implements the xxHash family (XXH3 64/128, XXH64, XXH32) in safe
//! Rust, with runtime SIMD dispatch for XXH3 on long inputs.
//!
//! ```
//! # #[cfg(feature = "xxh3")]
//! # {
//! let h64 = zenhash::xxh3_64(b"hello world");
//! let h128 = zenhash::xxh3_128(b"hello world");
//! assert_eq!(h64, 0xD447_B1EA_40E6_988B);
//! assert_eq!(h128, 0xDF8D_09E9_3F87_4900_A99B_8775_CC15_B6C7);
//! # }
//! ```
//!
//! The default-enabled `xxh3` feature provides XXH3 and its custom-secret errors.
//! XXH32 and XXH64 remain available without it. The goal is safe Rust with
//! competitive performance on x86, ARM and WebAssembly; performance parity
//! across ISAs remains a measurement goal. XXH3 is non-cryptographic: see the
//! [usage guidance](https://github.com/imazen/zenhash/blob/main/docs/xxh3-quality.md).
//!
//! Values are checked against two independent Rust implementations
//! (xxhash-rust and twox-hash) on every SIMD tier the test machine has.
#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "xxh3")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "xxh3")]
whereat::define_at_crate_info!();

mod common;
#[cfg(feature = "xxh3")]
mod error;
#[cfg(feature = "xxh3")]
mod xxh3;
mod xxh32;
mod xxh64;

#[cfg(feature = "xxh3")]
pub use error::Error;
#[cfg(feature = "xxh3")]
pub use xxh3::{
    XXH3_SECRET_SIZE_MIN, Xxh3, xxh3_64, xxh3_64_with_secret, xxh3_64_with_seed, xxh3_128,
    xxh3_128_with_secret, xxh3_128_with_seed,
};
pub use xxh32::{Xxh32, xxh32};
pub use xxh64::{Xxh64, xxh64};

/// Runs the README's code blocks as doctests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
