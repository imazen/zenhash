//! zenhash implements XXH3, classic xxHash and standard rapidhash v3 in safe
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
//! `rapidhash`, `xxh32` and `xxh64` are opt-in algorithm features. The goal is safe Rust with
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

#[cfg(any(
    feature = "xxh3",
    feature = "xxh32",
    feature = "xxh64",
    feature = "rapidhash"
))]
mod common;
#[cfg(feature = "fnv")]
mod fnv;
#[cfg(feature = "fnv")]
pub use fnv::{
    Fnv1a32, Fnv1a64, Fnv1a128, fnv1a32, fnv1a64, fnv1a64_fmix, fnv1a64_fold32, fnv1a128,
};
#[cfg(feature = "rapidhash")]
mod rapidhash;

#[cfg(feature = "rapidhash")]
pub use rapidhash::{
    RapidhashV3, rapidhash_v3, rapidhash_v3_with_seed, rapidhash_v3_with_seed_and_secret,
};
#[cfg(feature = "xxh3")]
mod error;
#[cfg(feature = "xxh3")]
mod xxh3;
#[cfg(feature = "xxh32")]
mod xxh32;
#[cfg(feature = "xxh64")]
mod xxh64;

#[cfg(feature = "xxh3")]
pub use error::Error;
#[cfg(feature = "xxh3")]
pub use xxh3::{
    XXH3_SECRET_SIZE_MIN, Xxh3, xxh3_64, xxh3_64_with_secret, xxh3_64_with_seed, xxh3_128,
    xxh3_128_with_secret, xxh3_128_with_seed,
};
#[cfg(feature = "xxh32")]
pub use xxh32::{Xxh32, xxh32};
#[cfg(feature = "xxh64")]
pub use xxh64::{Xxh64, xxh64};

/// Runs the README's code blocks as doctests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
