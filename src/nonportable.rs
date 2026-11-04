//! Hashers for trusted in-memory tables with target-dependent output.
//!
//! Enable `fxhash` for [`crate::nonportable::FxHasher`]. It wraps an exactly
//! pinned rustc-hash version behind zenhash's own type, without re-exporting
//! upstream types. It works without `std` or allocation.
//!
//! These hashers preserve typed `core::hash::Hasher` operations. Output may
//! differ by pointer width, target features and the sequence of write calls.
//! Adjacent writes are not concatenated, and empty writes can change the state.
//! Use the portable algorithms for persisted byte fingerprints or streaming I/O.
//!
//! ```
//! use core::hash::{BuildHasherDefault, Hash, Hasher};
//! use zenhash::nonportable::FxHasher;
//!
//! let mut hasher = FxHasher::new();
//! (42_u64, 7_u32).hash(&mut hasher);
//! let fingerprint = hasher.finish();
//! assert_eq!(fingerprint, hasher.finish());
//!
//! let mut table = std::collections::HashMap::<u64, &str, BuildHasherDefault<FxHasher>>::default();
//! table.insert(42, "answer");
//! assert_eq!(table[&42], "answer");
//! ```

use core::hash::Hasher;

/// FxHasher for trusted in-memory hash tables, backed by rustc-hash `=2.1.3`.
///
/// The backend is private. Every stable typed integer write is forwarded to
/// its corresponding upstream method, preserving upstream's integer fast paths.
/// `core::hash::Hasher` is also re-exported as `std::hash::Hasher`.
///
/// Results are not portable across targets or write schedules. A seed does not
/// provide cryptographic or adversarial hash-table protection. `Debug` omits
/// both the seed and accumulated state.
///
/// This is a typed hash-table adapter, so the `io` feature does not add
/// `std::io::Write`:
///
/// ```compile_fail
/// use std::io::Write;
/// let mut hasher = zenhash::nonportable::FxHasher::new();
/// hasher.write_all(b"hello").unwrap();
/// ```
#[derive(Clone)]
pub struct FxHasher {
    inner: rustc_hash::FxHasher,
}

impl FxHasher {
    /// Creates the pinned upstream algorithm with its default state (seed zero).
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: rustc_hash::FxHasher::default(),
        }
    }

    /// Creates the pinned upstream algorithm with a pointer-width-sized seed.
    #[inline]
    #[must_use]
    pub const fn with_seed(seed: usize) -> Self {
        Self {
            inner: rustc_hash::FxHasher::with_seed(seed),
        }
    }
}

impl Default for FxHasher {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for FxHasher {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FxHasher").finish_non_exhaustive()
    }
}

// Forward typed methods directly: Hasher's byte-based defaults have different
// semantics and would lose the upstream integer fast paths.
macro_rules! integer_writes {
    ($($method:ident($word:ty)),+ $(,)?) => {
        $(
            #[inline]
            fn $method(&mut self, value: $word) {
                self.inner.$method(value);
            }
        )+
    };
}

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.inner.write(bytes);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.inner.finish()
    }

    integer_writes!(
        write_u8(u8),
        write_u16(u16),
        write_u32(u32),
        write_u64(u64),
        write_u128(u128),
        write_usize(usize),
        write_i8(i8),
        write_i16(i16),
        write_i32(i32),
        write_i64(i64),
        write_i128(i128),
        write_isize(isize),
    );
}
