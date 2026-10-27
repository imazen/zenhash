//! Reference-compatible rapidhash v3 (standard, unprotected variant).
//!
//! Ported from Nicoshev/rapidhash, commit
//! 1ae7842f59f25929a338e5a76e3fa350673e48e1, rapidhash.h.
//! Copyright (c) 2025 Nicolas De Carli. The MIT notice is preserved in
//! LICENSE-RAPIDHASH and tests/reference/rapidhash.h.

use crate::common::{r32, r64};

const SECRET: [u64; 8] = [
    0x2d35_8dcc_aa6c_78a5,
    0x8bb8_4b93_962e_acc9,
    0x4b33_a62e_d433_d4a3,
    0x4d5a_2da5_1de1_aa47,
    0xa076_1d64_78bd_642f,
    0xe703_7ed1_a0b4_28db,
    0x90ed_1765_281c_388c,
    0xaaaa_aaaa_aaaa_aaaa,
];

#[inline(always)]
fn multiply(a: u64, b: u64) -> (u64, u64) {
    let product = u128::from(a) * u128::from(b);
    (product as u64, (product >> 64) as u64)
}

#[inline(always)]
fn mix(a: u64, b: u64) -> u64 {
    let (lo, hi) = multiply(a, b);
    lo ^ hi
}

#[inline]
fn init(seed: u64, secret: &[u64; 8]) -> u64 {
    seed ^ mix(seed ^ secret[2], secret[1])
}

#[inline]
fn consume(lanes: &mut [u64; 7], block: &[u8; 112], secret: &[u64; 8]) {
    for (i, lane) in lanes.iter_mut().enumerate() {
        *lane = mix(
            r64(block, i * 16) ^ secret[i],
            r64(block, i * 16 + 8) ^ *lane,
        );
    }
}

#[inline(always)]
fn finish(a: u64, b: u64, seed: u64, len: usize, secret: &[u64; 8]) -> u64 {
    let (a, b) = multiply(a ^ secret[1], b ^ seed);
    mix(a ^ secret[7], b ^ secret[1] ^ len as u64)
}

#[inline]
fn short(data: &[u8], mut seed: u64, secret: &[u64; 8]) -> u64 {
    let len = data.len();
    let (a, b) = if len >= 8 {
        seed ^= len as u64;
        (r64(data, 0), r64(data, len - 8))
    } else if len >= 4 {
        seed ^= len as u64;
        (u64::from(r32(data, 0)), u64::from(r32(data, len - 4)))
    } else if len > 0 {
        (
            (u64::from(data[0]) << 45) | u64::from(data[len - 1]),
            u64::from(data[len >> 1]),
        )
    } else {
        (0, 0)
    };
    finish(a, b, seed, len, secret)
}

// The final 16 bytes can overlap the preceding 112-byte block. `tail` is
// 1..=112 bytes, and `last` is the final 16 bytes of the entire input.
#[inline]
fn long_tail(tail: &[u8], last: &[u8], mut seed: u64, secret: &[u64; 8]) -> u64 {
    let len = tail.len();
    for (offset, key) in [(0, 2), (16, 2), (32, 1), (48, 1), (64, 2), (80, 1)] {
        if len > offset + 16 {
            seed = mix(
                r64(tail, offset) ^ secret[key],
                r64(tail, offset + 8) ^ seed,
            );
        }
    }
    finish(r64(last, 0) ^ len as u64, r64(last, 8), seed, len, secret)
}

/// Computes standard rapidhash v3 with seed 0 and the reference default secrets.
///
/// The output matches the C reference's `rapidhash`. This is a non-cryptographic
/// hash; default secrets and a seed do not provide HashDoS protection.
///
/// ```
/// assert_eq!(zenhash::rapidhash_v3(b"hello"),
///            zenhash::rapidhash_v3_with_seed(b"hello", 0));
/// ```
#[must_use]
#[inline]
pub fn rapidhash_v3(data: &[u8]) -> u64 {
    rapidhash_v3_with_seed(data, 0)
}

/// Computes standard rapidhash v3 with `seed` and the reference default secrets.
///
/// Matches C's `rapidhash_withSeed`; this does not generate randomized secrets.
#[must_use]
#[inline]
pub fn rapidhash_v3_with_seed(data: &[u8], seed: u64) -> u64 {
    rapidhash_v3_with_seed_and_secret(data, seed, &SECRET)
}

/// Computes standard rapidhash v3 with an explicit seed and eight secret words.
///
/// Matches C's `rapidhash_internal`. Words are numeric values, not native-endian
/// byte encodings. Use well-randomized secrets; this remains non-cryptographic.
#[must_use]
#[inline]
pub fn rapidhash_v3_with_seed_and_secret(data: &[u8], seed: u64, secret: &[u64; 8]) -> u64 {
    let seed = init(seed, secret);
    if data.len() <= 16 {
        return short(data, seed, secret);
    }
    let mut lanes = [seed; 7];
    let mut tail = data;
    while tail.len() > 112 {
        let (block, rest) = tail.split_first_chunk::<112>().unwrap();
        consume(&mut lanes, block, secret);
        tail = rest;
    }
    let seed = if data.len() > 112 {
        lanes.into_iter().fold(0, |a, b| a ^ b)
    } else {
        seed
    };
    long_tail(tail, &data[data.len() - 16..], seed, secret)
}

/// Streaming standard rapidhash v3. Every split produces the one-shot result.
///
/// Keeps bounded state without allocating. Digests do not consume or reset it.
/// Integer `Hasher` writes use native-endian bytes; feed explicit bytes with
/// [`update`](Self::update) for portable hashes.
///
/// ```
/// let mut hasher = zenhash::RapidhashV3::with_seed(7);
/// hasher.update(b"hello ");
/// hasher.update(b"world");
/// assert_eq!(hasher.digest(), zenhash::rapidhash_v3_with_seed(b"hello world", 7));
/// ```
#[derive(Clone)]
pub struct RapidhashV3 {
    lanes: [u64; 7],
    seed: u64,
    secret: [u64; 8],
    total_len: u64,
    // Previous block's final 16 bytes, then at most 112 unconsumed bytes.
    buf: [u8; 128],
    buf_len: usize,
    long: bool,
}

impl RapidhashV3 {
    /// Creates a hasher with seed 0 and the reference default secrets.
    pub fn new() -> Self {
        Self::with_seed(0)
    }

    /// Creates a hasher with `seed` and the reference default secrets.
    pub fn with_seed(seed: u64) -> Self {
        Self::with_seed_and_secret(seed, SECRET)
    }

    /// Creates a hasher with an explicit seed and eight secret words, copied into state.
    pub fn with_seed_and_secret(seed: u64, secret: [u64; 8]) -> Self {
        Self {
            lanes: [init(seed, &secret); 7],
            seed,
            secret,
            total_len: 0,
            buf: [0; 128],
            buf_len: 0,
            long: false,
        }
    }

    /// Feeds bytes into the hash. Empty updates leave the state unchanged.
    pub fn update(&mut self, mut data: &[u8]) {
        self.total_len = self.total_len.wrapping_add(data.len() as u64);
        if self.buf_len > 0 {
            let take = (112 - self.buf_len).min(data.len());
            self.buf[16 + self.buf_len..16 + self.buf_len + take].copy_from_slice(&data[..take]);
            self.buf_len += take;
            data = &data[take..];
            if data.is_empty() {
                return;
            }
            consume(
                &mut self.lanes,
                self.buf[16..].split_first_chunk::<112>().unwrap().0,
                &self.secret,
            );
            self.buf.copy_within(112..128, 0);
            self.buf_len = 0;
            self.long = true;
        }
        while data.len() > 112 {
            let (block, rest) = data.split_first_chunk::<112>().unwrap();
            consume(&mut self.lanes, block, &self.secret);
            self.buf[..16].copy_from_slice(&block[96..]);
            self.long = true;
            data = rest;
        }
        self.buf[16..16 + data.len()].copy_from_slice(data);
        self.buf_len = data.len();
    }

    /// Returns the hash of all bytes fed so far, keeping the state intact.
    #[must_use]
    pub fn digest(&self) -> u64 {
        let tail = &self.buf[16..16 + self.buf_len];
        if !self.long && self.buf_len <= 16 {
            short(tail, self.lanes[0], &self.secret)
        } else {
            let seed = if self.long {
                self.lanes.iter().fold(0, |a, b| a ^ b)
            } else {
                self.lanes[0]
            };
            long_tail(
                tail,
                &self.buf[self.buf_len..self.buf_len + 16],
                seed,
                &self.secret,
            )
        }
    }

    /// Resets the hasher, preserving its seed.
    pub fn reset(&mut self) {
        *self = Self::with_seed_and_secret(self.seed, self.secret);
    }
}

impl Default for RapidhashV3 {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for RapidhashV3 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RapidhashV3")
            .field("total_len", &self.total_len)
            .finish_non_exhaustive()
    }
}

impl core::hash::Hasher for RapidhashV3 {
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
impl std::io::Write for RapidhashV3 {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
