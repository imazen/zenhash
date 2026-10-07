//! XXH64: the 64-bit classic xxHash.

use crate::common::{PRIME64_1, PRIME64_2, PRIME64_3, PRIME64_4, PRIME64_5, fill, r32, r64};

#[inline(always)]
fn round(acc: u64, input: u64) -> u64 {
    acc.wrapping_add(input.wrapping_mul(PRIME64_2))
        .rotate_left(31)
        .wrapping_mul(PRIME64_1)
}

#[inline(always)]
fn merge_round(acc: u64, val: u64) -> u64 {
    (acc ^ round(0, val))
        .wrapping_mul(PRIME64_1)
        .wrapping_add(PRIME64_4)
}

#[inline(always)]
fn init_lanes(seed: u64) -> [u64; 4] {
    [
        seed.wrapping_add(PRIME64_1).wrapping_add(PRIME64_2),
        seed.wrapping_add(PRIME64_2),
        seed,
        seed.wrapping_sub(PRIME64_1),
    ]
}

#[inline(always)]
fn consume(v: &mut [u64; 4], stripes: &[[u8; 32]]) {
    for s in stripes {
        for (i, lane) in v.iter_mut().enumerate() {
            *lane = round(*lane, r64(s, i * 8));
        }
    }
}

#[inline(always)]
fn merge_lanes(v: &[u64; 4]) -> u64 {
    let mut h = v[0]
        .rotate_left(1)
        .wrapping_add(v[1].rotate_left(7))
        .wrapping_add(v[2].rotate_left(12))
        .wrapping_add(v[3].rotate_left(18));
    for &lane in v {
        h = merge_round(h, lane);
    }
    h
}

/// Mixes the last `< 32` bytes into `h` and avalanches.
#[inline(always)]
fn finalize(mut h: u64, rest: &[u8]) -> u64 {
    let (words, rest) = rest.as_chunks::<8>();
    for w in words {
        h ^= round(0, u64::from_le_bytes(*w));
        h = h
            .rotate_left(27)
            .wrapping_mul(PRIME64_1)
            .wrapping_add(PRIME64_4);
    }
    let rest = if rest.len() >= 4 {
        h ^= u64::from(r32(rest, 0)).wrapping_mul(PRIME64_1);
        h = h
            .rotate_left(23)
            .wrapping_mul(PRIME64_2)
            .wrapping_add(PRIME64_3);
        &rest[4..]
    } else {
        rest
    };
    for &b in rest {
        h ^= u64::from(b).wrapping_mul(PRIME64_5);
        h = h.rotate_left(11).wrapping_mul(PRIME64_1);
    }
    h ^= h >> 33;
    h = h.wrapping_mul(PRIME64_2);
    h ^= h >> 29;
    h = h.wrapping_mul(PRIME64_3);
    h ^ (h >> 32)
}

/// Computes the XXH64 hash of `data` with `seed`.
///
/// ```
/// assert_eq!(zenhash::xxh64(b"", 0), 0xEF46_DB37_51D8_E999);
/// ```
#[must_use]
pub fn xxh64(data: &[u8], seed: u64) -> u64 {
    let (stripes, rest) = data.as_chunks::<32>();
    let h = if stripes.is_empty() {
        seed.wrapping_add(PRIME64_5)
    } else {
        let mut v = init_lanes(seed);
        consume(&mut v, stripes);
        merge_lanes(&v)
    };
    finalize(h.wrapping_add(data.len() as u64), rest)
}

/// Streaming XXH64. Produces the same value as [`xxh64`] for the concatenation
/// of everything passed to [`update`](Self::update).
///
/// ```
/// let mut h = zenhash::Xxh64::with_seed(7);
/// h.update(b"hello ");
/// h.update(b"world");
/// assert_eq!(h.digest(), zenhash::xxh64(b"hello world", 7));
/// ```
#[derive(Clone)]
pub struct Xxh64 {
    v: [u64; 4],
    total_len: u64,
    seed: u64,
    buf: [u8; 32],
    buf_len: usize,
}

impl Xxh64 {
    /// Creates a hasher with seed 0.
    pub fn new() -> Self {
        Self::with_seed(0)
    }

    /// Creates a hasher with `seed`.
    pub fn with_seed(seed: u64) -> Self {
        Self {
            v: init_lanes(seed),
            total_len: 0,
            seed,
            buf: [0; 32],
            buf_len: 0,
        }
    }

    /// Feeds `data` into the hash.
    pub fn update(&mut self, data: &[u8]) {
        self.total_len = self.total_len.wrapping_add(data.len() as u64);
        let mut data = data;
        if self.buf_len > 0 {
            data = fill(&mut self.buf, &mut self.buf_len, data);
            if self.buf_len < 32 {
                return;
            }
            consume(&mut self.v, &[self.buf]);
            self.buf_len = 0;
        }
        let (stripes, rest) = data.as_chunks::<32>();
        consume(&mut self.v, stripes);
        self.buf[..rest.len()].copy_from_slice(rest);
        self.buf_len = rest.len();
    }

    /// Returns the hash of everything fed so far. Does not reset the state.
    #[must_use]
    pub fn digest(&self) -> u64 {
        let h = if self.total_len >= 32 {
            merge_lanes(&self.v)
        } else {
            self.seed.wrapping_add(PRIME64_5)
        };
        finalize(h.wrapping_add(self.total_len), &self.buf[..self.buf_len])
    }

    /// Resets to the state [`with_seed`](Self::with_seed) produced, keeping the seed.
    pub fn reset(&mut self) {
        *self = Self::with_seed(self.seed);
    }
}

impl Default for Xxh64 {
    /// Seed 0, same as [`new`](Self::new).
    fn default() -> Self {
        Self::new()
    }
}

/// Shows the byte count only: the seed and buffered input stay out of logs.
impl core::fmt::Debug for Xxh64 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Xxh64")
            .field("total_len", &self.total_len)
            .finish_non_exhaustive()
    }
}

/// `Hasher::write_u32` and the other integer methods feed the integer's
/// native-endian bytes, and `Hash` impls for slices and strings add a length
/// prefix of platform-dependent width. For hashes that must match across
/// platforms or other xxHash implementations, feed bytes with `write`/`update`.
impl core::hash::Hasher for Xxh64 {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.update(bytes);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.digest()
    }
}

#[cfg(feature = "std")]
impl std::io::Write for Xxh64 {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.update(buf);
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
