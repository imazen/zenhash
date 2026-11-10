// Ported from Austin Appleby's public-domain MurmurHash3_x64_128 reference:
// https://github.com/aappleby/smhasher/blob/07bb4de10a63e8cc2e1724865454eba635742383/src/MurmurHash3.cpp
// MurmurHash3 was written by Austin Appleby, and is placed in the public
// domain. The author hereby disclaims copyright to this source code.

use crate::common::{fmix64, r64};

const C1: u64 = 0x87c3_7b91_1142_53d5;
const C2: u64 = 0x4cf5_ad43_2745_937f;

const fn mix1(k: u64) -> u64 {
    k.wrapping_mul(C1).rotate_left(31).wrapping_mul(C2)
}
const fn mix2(k: u64) -> u64 {
    k.wrapping_mul(C2).rotate_left(33).wrapping_mul(C1)
}

/// Canonical MurmurHash3_x64_128 with explicit seed and little-endian words.
///
/// The algorithm is fixed on every ISA, including 32-bit x86. It does not
/// become the incompatible x86_128 algorithm there. The low 64 bits are the
/// reference's first output word (`h1`); the high bits are its second (`h2`).
/// `to_le_bytes()` therefore gives `h1_le || h2_le` on every target.
#[must_use]
pub const fn murmur3_x64_128(data: &[u8], seed: u32) -> u128 {
    let mut state = Murmur3X64_128::with_seed(seed);
    state.update(data);
    state.digest()
}

/// Allocation-free, const streaming MurmurHash3_x64_128 on every ISA.
///
/// Input words are little-endian, arithmetic wraps at 64 bits, and length is
/// folded modulo 2^64. Empty updates, alignment and update boundaries do not
/// affect the result. This always selects x64_128, even on 32-bit x86.
#[derive(Clone)]
pub struct Murmur3X64_128 {
    h1: u64,
    h2: u64,
    len: u64,
    tail: [u8; 16],
    tail_len: u8,
}

impl Murmur3X64_128 {
    /// Initialize both accumulators with the zero-extended 32-bit seed.
    #[must_use]
    pub const fn with_seed(seed: u32) -> Self {
        Self {
            h1: seed as u64,
            h2: seed as u64,
            len: 0,
            tail: [0; 16],
            tail_len: 0,
        }
    }

    const fn block(&mut self, k1: u64, k2: u64) {
        self.h1 ^= mix1(k1);
        self.h1 = self
            .h1
            .rotate_left(27)
            .wrapping_add(self.h2)
            .wrapping_mul(5)
            .wrapping_add(0x52dc_e729);
        self.h2 ^= mix2(k2);
        self.h2 = self
            .h2
            .rotate_left(31)
            .wrapping_add(self.h1)
            .wrapping_mul(5)
            .wrapping_add(0x3849_5ab5);
    }

    /// Append bytes without allocation. Empty updates have no effect.
    pub const fn update(&mut self, data: &[u8]) {
        self.len = self.len.wrapping_add(data.len() as u64);
        let mut i = 0;
        while self.tail_len != 0 && i < data.len() {
            self.tail[self.tail_len as usize] = data[i];
            self.tail_len += 1;
            i += 1;
            if self.tail_len == 16 {
                self.block(r64(&self.tail, 0), r64(&self.tail, 8));
                self.tail_len = 0;
            }
        }
        while data.len() - i >= 16 {
            self.block(r64(data, i), r64(data, i + 8));
            i += 16;
        }
        while i < data.len() {
            self.tail[self.tail_len as usize] = data[i];
            self.tail_len += 1;
            i += 1;
        }
    }

    /// Snapshot the full 128-bit digest without consuming or changing state.
    ///
    /// Low word = reference `h1`, high word = reference `h2`.
    #[must_use]
    pub const fn digest(&self) -> u128 {
        let mut h1 = self.h1;
        let mut h2 = self.h2;
        let mut k1 = 0;
        let mut k2 = 0;
        let mut i = 0;
        while i < self.tail_len as usize {
            if i < 8 {
                k1 |= (self.tail[i] as u64) << (i * 8);
            } else {
                k2 |= (self.tail[i] as u64) << ((i - 8) * 8);
            }
            i += 1;
        }
        if self.tail_len > 8 {
            h2 ^= mix2(k2);
        }
        if self.tail_len > 0 {
            h1 ^= mix1(k1);
        }
        h1 ^= self.len;
        h2 ^= self.len;
        h1 = h1.wrapping_add(h2);
        h2 = h2.wrapping_add(h1);
        h1 = fmix64(h1);
        h2 = fmix64(h2);
        h1 = h1.wrapping_add(h2);
        h2 = h2.wrapping_add(h1);
        (h1 as u128) | ((h2 as u128) << 64)
    }
}

impl core::fmt::Debug for Murmur3X64_128 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Murmur3X64_128")
            .field("bytes_mod_2_64", &self.len)
            .finish_non_exhaustive()
    }
}
