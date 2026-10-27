//! Stable FNV-1a fingerprints. These preserve existing identifiers; they are
//! not cryptographic hashes. Start with XXH3 for new non-adversarial byte fingerprints;
//! benchmark rapidhash v3 for short native keys.

macro_rules! fnv {
    ($name:ident, $function:ident, $word:ty, $basis:expr, $prime:expr) => {
        /// FNV-1a streaming state. Input is always interpreted as bytes.
        /// `Hasher::finish` returns the low 64 bits for a 128-bit state.
        #[derive(Clone)]
        pub struct $name {
            state: $word,
            basis: $word,
        }
        impl $name {
            /// Starts with the standard FNV offset basis.
            #[must_use]
            pub const fn new() -> Self {
                Self::with_basis($basis)
            }
            /// Starts with a caller-supplied offset basis. Passing an earlier
            /// digest continues that byte stream. This is not a keyed hash.
            #[must_use]
            pub const fn with_basis(basis: $word) -> Self {
                Self {
                    state: basis,
                    basis,
                }
            }
            /// Appends bytes without allocating.
            pub fn update(&mut self, bytes: &[u8]) {
                for &byte in bytes {
                    self.state = (self.state ^ <$word>::from(byte)).wrapping_mul($prime);
                }
            }
            /// Returns the current digest without consuming or changing state.
            #[must_use]
            pub const fn digest(&self) -> $word {
                self.state
            }
            /// Restores the original offset basis.
            pub fn reset(&mut self) {
                self.state = self.basis;
            }
        }
        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct(stringify!($name)).finish_non_exhaustive()
            }
        }
        impl core::hash::Hasher for $name {
            fn write(&mut self, bytes: &[u8]) {
                self.update(bytes);
            }
            fn finish(&self) -> u64 {
                self.state as u64
            }
        }
        #[cfg(feature = "io")]
        impl std::io::Write for $name {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.update(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        /// Hashes bytes with the standard FNV-1a offset basis.
        #[must_use]
        pub fn $function(bytes: &[u8]) -> $word {
            let mut h = $name::new();
            h.update(bytes);
            h.digest()
        }
    };
}
fnv!(Fnv1a32, fnv1a32, u32, 0x811c9dc5, 0x01000193);
fnv!(Fnv1a64, fnv1a64, u64, 0xcbf29ce484222325, 0x100000001b3);
fnv!(
    Fnv1a128,
    fnv1a128,
    u128,
    0x6c62272e07bb014262b821756295c58d,
    0x0000000001000000000000000000013b
);

impl Fnv1a64 {
    /// XOR-folds the two halves, preserving zensim's 32-bit identifier formula.
    #[must_use]
    pub const fn digest_fold32(&self) -> u32 {
        (self.state ^ (self.state >> 32)) as u32
    }
    /// Applies Murmur3 fmix64 to the current FNV digest, preserving zensr's
    /// fingerprint formula. The streaming state remains unfinalized.
    #[must_use]
    pub const fn digest_fmix(&self) -> u64 {
        let h = (self.state ^ (self.state >> 33)).wrapping_mul(0xff51afd7ed558ccd);
        let h = (h ^ (h >> 33)).wrapping_mul(0xc4ceb9fe1a85ec53);
        h ^ (h >> 33)
    }
}

/// Standard FNV-1a64 followed by XOR-folding to 32 bits.
#[must_use]
pub fn fnv1a64_fold32(bytes: &[u8]) -> u32 {
    let mut h = Fnv1a64::new();
    h.update(bytes);
    h.digest_fold32()
}

/// Standard FNV-1a64 followed by Murmur3 fmix64 (not a cryptographic hash).
#[must_use]
pub fn fnv1a64_fmix(bytes: &[u8]) -> u64 {
    let mut h = Fnv1a64::new();
    h.update(bytes);
    h.digest_fmix()
}
