//! XXH32: the 32-bit classic xxHash.

use crate::common::{PRIME32_1, PRIME32_2, PRIME32_3, PRIME32_4, PRIME32_5, fill, r32};

#[inline(always)]
fn round(acc: u32, input: u32) -> u32 {
    acc.wrapping_add(input.wrapping_mul(PRIME32_2))
        .rotate_left(13)
        .wrapping_mul(PRIME32_1)
}

#[inline(always)]
fn init_lanes(seed: u32) -> [u32; 4] {
    [
        seed.wrapping_add(PRIME32_1).wrapping_add(PRIME32_2),
        seed.wrapping_add(PRIME32_2),
        seed,
        seed.wrapping_sub(PRIME32_1),
    ]
}

#[inline(always)]
fn consume(v: &mut [u32; 4], stripes: &[[u8; 16]]) {
    for s in stripes {
        for (i, lane) in v.iter_mut().enumerate() {
            *lane = round(*lane, r32(s, i * 4));
        }
    }
}

#[inline(always)]
fn merge_lanes(v: &[u32; 4]) -> u32 {
    v[0].rotate_left(1)
        .wrapping_add(v[1].rotate_left(7))
        .wrapping_add(v[2].rotate_left(12))
        .wrapping_add(v[3].rotate_left(18))
}

/// Mixes the last `< 16` bytes into `h` and avalanches.
#[inline(always)]
fn finalize(mut h: u32, rest: &[u8]) -> u32 {
    let (words, rest) = rest.as_chunks::<4>();
    for w in words {
        h = h.wrapping_add(u32::from_le_bytes(*w).wrapping_mul(PRIME32_3));
        h = h.rotate_left(17).wrapping_mul(PRIME32_4);
    }
    for &b in rest {
        h = h.wrapping_add(u32::from(b).wrapping_mul(PRIME32_5));
        h = h.rotate_left(11).wrapping_mul(PRIME32_1);
    }
    h ^= h >> 15;
    h = h.wrapping_mul(PRIME32_2);
    h ^= h >> 13;
    h = h.wrapping_mul(PRIME32_3);
    h ^ (h >> 16)
}

/// Computes the XXH32 hash of `data` with `seed`.
///
/// ```
/// assert_eq!(zenhash::xxh32(b"", 0), 0x02CC_5D05);
/// ```
pub fn xxh32(data: &[u8], seed: u32) -> u32 {
    let (stripes, rest) = data.as_chunks::<16>();
    let h = if stripes.is_empty() {
        seed.wrapping_add(PRIME32_5)
    } else {
        let mut v = init_lanes(seed);
        consume(&mut v, stripes);
        merge_lanes(&v)
    };
    // XXH32 folds the length in modulo 2^32 by definition.
    finalize(h.wrapping_add(data.len() as u32), rest)
}

/// Streaming XXH32. Produces the same value as [`xxh32`] for the concatenation
/// of everything passed to [`update`](Self::update).
///
/// ```
/// let mut h = zenhash::Xxh32::with_seed(7);
/// h.update(b"hello ");
/// h.update(b"world");
/// assert_eq!(h.digest(), zenhash::xxh32(b"hello world", 7));
/// ```
#[derive(Clone, Debug)]
pub struct Xxh32 {
    v: [u32; 4],
    total_len: u64,
    seed: u32,
    buf: [u8; 16],
    buf_len: usize,
}

impl Xxh32 {
    /// Creates a hasher with `seed`.
    pub fn with_seed(seed: u32) -> Self {
        Self {
            v: init_lanes(seed),
            total_len: 0,
            seed,
            buf: [0; 16],
            buf_len: 0,
        }
    }

    /// Feeds `data` into the hash.
    pub fn update(&mut self, data: &[u8]) {
        self.total_len = self.total_len.wrapping_add(data.len() as u64);
        let mut data = data;
        if self.buf_len > 0 {
            data = fill(&mut self.buf, &mut self.buf_len, data);
            if self.buf_len < 16 {
                return;
            }
            consume(&mut self.v, &[self.buf]);
            self.buf_len = 0;
        }
        let (stripes, rest) = data.as_chunks::<16>();
        consume(&mut self.v, stripes);
        self.buf[..rest.len()].copy_from_slice(rest);
        self.buf_len = rest.len();
    }

    /// Returns the hash of everything fed so far. Does not reset the state.
    pub fn digest(&self) -> u32 {
        let h = if self.total_len >= 16 {
            merge_lanes(&self.v)
        } else {
            self.seed.wrapping_add(PRIME32_5)
        };
        finalize(
            h.wrapping_add(self.total_len as u32),
            &self.buf[..self.buf_len],
        )
    }

    /// Resets to the state [`with_seed`](Self::with_seed) produced, keeping the seed.
    pub fn reset(&mut self) {
        *self = Self::with_seed(self.seed);
    }
}

impl Default for Xxh32 {
    /// Seed 0.
    fn default() -> Self {
        Self::with_seed(0)
    }
}

impl core::hash::Hasher for Xxh32 {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.update(bytes);
    }

    /// The 32-bit digest, zero-extended.
    #[inline]
    fn finish(&self) -> u64 {
        u64::from(self.digest())
    }
}
