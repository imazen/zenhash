//! XXH3, 64- and 128-bit.
//!
//! Inputs of at most 240 bytes take short, branchy scalar paths that read the
//! first 136 bytes of the secret. Longer inputs run the stripe accumulator in
//! [`accumulate`], which dispatches once per call to the best SIMD tier.

mod accumulate;
mod stream;

use alloc::boxed::Box;

use whereat::{At, at};

use crate::Error;
use crate::common::{PRIME32_1, PRIME32_2, PRIME32_3, PRIME64_1, PRIME64_2, PRIME64_3, PRIME64_4};
use crate::common::{PRIME64_5, r32, r64};

pub use stream::Xxh3;

/// Minimum length of a custom XXH3 secret, in bytes.
pub const XXH3_SECRET_SIZE_MIN: usize = 136;

/// Length of the default XXH3 secret and of the secrets derived from a seed.
pub(crate) const SECRET_DEFAULT_SIZE: usize = 192;

pub(crate) const STRIPE_LEN: usize = 64;
pub(crate) const MID_SIZE_MAX: usize = 240;
pub(crate) const SECRET_CONSUME_RATE: usize = 8;
const SECRET_MERGEACCS_START: usize = 11;
pub(crate) const SECRET_LASTACC_START: usize = 7;
const MIDSIZE_STARTOFFSET: usize = 3;
const MIDSIZE_LASTOFFSET: usize = 17;

const PRIME_MX1: u64 = 0x1656_6791_9E37_79F9;
const PRIME_MX2: u64 = 0x9FB2_1C65_1E98_DF25;

/// The default secret from the xxHash specification.
pub(crate) const DEFAULT_SECRET: [u8; SECRET_DEFAULT_SIZE] = [
    0xb8, 0xfe, 0x6c, 0x39, 0x23, 0xa4, 0x4b, 0xbe, 0x7c, 0x01, 0x81, 0x2c, 0xf7, 0x21, 0xad, 0x1c,
    0xde, 0xd4, 0x6d, 0xe9, 0x83, 0x90, 0x97, 0xdb, 0x72, 0x40, 0xa4, 0xa4, 0xb7, 0xb3, 0x67, 0x1f,
    0xcb, 0x79, 0xe6, 0x4e, 0xcc, 0xc0, 0xe5, 0x78, 0x82, 0x5a, 0xd0, 0x7d, 0xcc, 0xff, 0x72, 0x21,
    0xb8, 0x08, 0x46, 0x74, 0xf7, 0x43, 0x24, 0x8e, 0xe0, 0x35, 0x90, 0xe6, 0x81, 0x3a, 0x26, 0x4c,
    0x3c, 0x28, 0x52, 0xbb, 0x91, 0xc3, 0x00, 0xcb, 0x88, 0xd0, 0x65, 0x8b, 0x1b, 0x53, 0x2e, 0xa3,
    0x71, 0x64, 0x48, 0x97, 0xa2, 0x0d, 0xf9, 0x4e, 0x38, 0x19, 0xef, 0x46, 0xa9, 0xde, 0xac, 0xd8,
    0xa8, 0xfa, 0x76, 0x3f, 0xe3, 0x9c, 0x34, 0x3f, 0xf9, 0xdc, 0xbb, 0xc7, 0xc7, 0x0b, 0x4f, 0x1d,
    0x8a, 0x51, 0xe0, 0x4b, 0xcd, 0xb4, 0x59, 0x31, 0xc8, 0x9f, 0x7e, 0xc9, 0xd9, 0x78, 0x73, 0x64,
    0xea, 0xc5, 0xac, 0x83, 0x34, 0xd3, 0xeb, 0xc3, 0xc5, 0x81, 0xa0, 0xff, 0xfa, 0x13, 0x63, 0xeb,
    0x17, 0x0d, 0xdd, 0x51, 0xb7, 0xf0, 0xda, 0x49, 0xd3, 0x16, 0x55, 0x26, 0x29, 0xd4, 0x68, 0x9e,
    0x2b, 0x16, 0xbe, 0x58, 0x7d, 0x47, 0xa1, 0xfc, 0x8f, 0xf8, 0xb8, 0xd1, 0x7a, 0xd0, 0x31, 0xce,
    0x45, 0xcb, 0x3a, 0x8f, 0x95, 0x16, 0x04, 0x28, 0xaf, 0xd7, 0xfb, 0xca, 0xbb, 0x4b, 0x40, 0x7e,
];

/// The secret prefix the short paths read.
type ShortSecret = [u8; XXH3_SECRET_SIZE_MIN];

#[inline(always)]
fn short_secret(secret: &[u8]) -> Option<&ShortSecret> {
    secret.first_chunk::<XXH3_SECRET_SIZE_MIN>()
}

#[inline(always)]
fn default_short_secret() -> &'static ShortSecret {
    // 192 >= 136, so this is a constant-folded split.
    DEFAULT_SECRET
        .first_chunk::<XXH3_SECRET_SIZE_MIN>()
        .unwrap_or(&[0; 136])
}

/// Checks a caller-supplied secret.
pub(crate) fn validate_secret(secret: &[u8]) -> Result<&ShortSecret, At<Error>> {
    short_secret(secret).ok_or_else(|| at!(Error::SecretTooShort { len: secret.len() }))
}

/// Derives the secret XXH3 uses for long inputs with a non-zero seed.
pub(crate) fn derive_secret(seed: u64) -> [u8; SECRET_DEFAULT_SIZE] {
    let mut out = DEFAULT_SECRET;
    let (pairs, _) = out.as_chunks_mut::<16>();
    for pair in pairs {
        let lo = u64::from_le_bytes(pair[..8].try_into().unwrap_or([0; 8])).wrapping_add(seed);
        let hi = u64::from_le_bytes(pair[8..].try_into().unwrap_or([0; 8])).wrapping_sub(seed);
        pair[..8].copy_from_slice(&lo.to_le_bytes());
        pair[8..].copy_from_slice(&hi.to_le_bytes());
    }
    out
}

// ---------------------------------------------------------------------------
// Mixing primitives

#[inline(always)]
fn mul128_fold64(a: u64, b: u64) -> u64 {
    let p = u128::from(a) * u128::from(b);
    (p as u64) ^ ((p >> 64) as u64)
}

#[inline(always)]
fn mult64to128(a: u64, b: u64) -> (u64, u64) {
    let p = u128::from(a) * u128::from(b);
    (p as u64, (p >> 64) as u64)
}

#[inline(always)]
fn xorshift64(v: u64, shift: u32) -> u64 {
    v ^ (v >> shift)
}

#[inline(always)]
pub(crate) fn avalanche(h: u64) -> u64 {
    xorshift64(xorshift64(h, 37).wrapping_mul(PRIME_MX1), 32)
}

#[inline(always)]
fn xxh64_avalanche(mut h: u64) -> u64 {
    h ^= h >> 33;
    h = h.wrapping_mul(PRIME64_2);
    h ^= h >> 29;
    h = h.wrapping_mul(PRIME64_3);
    h ^ (h >> 32)
}

#[inline(always)]
fn rrmxmx(mut h: u64, len: u64) -> u64 {
    h ^= h.rotate_left(49) ^ h.rotate_left(24);
    h = h.wrapping_mul(PRIME_MX2);
    h ^= (h >> 35).wrapping_add(len);
    h = h.wrapping_mul(PRIME_MX2);
    xorshift64(h, 28)
}

#[inline(always)]
fn mix16(input: &[u8], i: usize, secret: &[u8], s: usize, seed: u64) -> u64 {
    let lo = r64(input, i);
    let hi = r64(input, i + 8);
    mul128_fold64(
        lo ^ r64(secret, s).wrapping_add(seed),
        hi ^ r64(secret, s + 8).wrapping_sub(seed),
    )
}

// ---------------------------------------------------------------------------
// 64-bit short paths (len <= 240)

#[inline(always)]
fn len_1to3_64(input: &[u8], secret: &ShortSecret, seed: u64) -> u64 {
    let len = input.len();
    let c1 = u32::from(input[0]);
    let c2 = u32::from(input[len >> 1]);
    let c3 = u32::from(input[len - 1]);
    let combined = (c1 << 16) | (c2 << 24) | c3 | ((len as u32) << 8);
    let bitflip = u64::from(r32(secret, 0) ^ r32(secret, 4)).wrapping_add(seed);
    xxh64_avalanche(u64::from(combined) ^ bitflip)
}

#[inline(always)]
fn len_4to8_64(input: &[u8], secret: &ShortSecret, mut seed: u64) -> u64 {
    let len = input.len();
    seed ^= u64::from((seed as u32).swap_bytes()) << 32;
    let input1 = r32(input, 0);
    let input2 = r32(input, len - 4);
    let bitflip = (r64(secret, 8) ^ r64(secret, 16)).wrapping_sub(seed);
    let input64 = u64::from(input2).wrapping_add(u64::from(input1) << 32);
    rrmxmx(input64 ^ bitflip, len as u64)
}

#[inline(always)]
fn len_9to16_64(input: &[u8], secret: &ShortSecret, seed: u64) -> u64 {
    let len = input.len();
    let bitflip1 = (r64(secret, 24) ^ r64(secret, 32)).wrapping_add(seed);
    let bitflip2 = (r64(secret, 40) ^ r64(secret, 48)).wrapping_sub(seed);
    let lo = r64(input, 0) ^ bitflip1;
    let hi = r64(input, len - 8) ^ bitflip2;
    let acc = (len as u64)
        .wrapping_add(lo.swap_bytes())
        .wrapping_add(hi)
        .wrapping_add(mul128_fold64(lo, hi));
    avalanche(acc)
}

#[inline(always)]
fn len_0to16_64(input: &[u8], secret: &ShortSecret, seed: u64) -> u64 {
    match input.len() {
        9.. => len_9to16_64(input, secret, seed),
        4.. => len_4to8_64(input, secret, seed),
        1.. => len_1to3_64(input, secret, seed),
        0 => xxh64_avalanche(seed ^ r64(secret, 56) ^ r64(secret, 64)),
    }
}

#[inline(always)]
fn len_17to128_64(input: &[u8], secret: &ShortSecret, seed: u64) -> u64 {
    let len = input.len();
    let mut acc = (len as u64).wrapping_mul(PRIME64_1);
    if len > 32 {
        if len > 64 {
            if len > 96 {
                acc = acc.wrapping_add(mix16(input, 48, secret, 96, seed));
                acc = acc.wrapping_add(mix16(input, len - 64, secret, 112, seed));
            }
            acc = acc.wrapping_add(mix16(input, 32, secret, 64, seed));
            acc = acc.wrapping_add(mix16(input, len - 48, secret, 80, seed));
        }
        acc = acc.wrapping_add(mix16(input, 16, secret, 32, seed));
        acc = acc.wrapping_add(mix16(input, len - 32, secret, 48, seed));
    }
    acc = acc.wrapping_add(mix16(input, 0, secret, 0, seed));
    acc = acc.wrapping_add(mix16(input, len - 16, secret, 16, seed));
    avalanche(acc)
}

/// [`mix16`] on a fixed 16-byte chunk; with constant `s` the secret reads fold.
/// [`mix16`] on a fixed 16-byte chunk; with constant `s` the secret reads fold.
///
/// `U128_KEY` picks how the key xor is written, and each mid-size caller uses
/// the form LLVM compiles to scalar code (see CLAUDE.md):
/// - seed 0 (`false`): two u64 xors with immediates. As one u128 xor against
///   a constant, LLVM emits `pxor` plus lane extracts instead.
/// - runtime seed (`true`): one u128 xor. With two u64 xors, the SLP
///   vectorizer packs the `(s + seed, s - seed)` pairs into SSE registers; it
///   runs before i128 is split into two registers, so a u128 gives it no
///   pair to pack.
#[inline(always)]
fn mix16_chunk<const U128_KEY: bool>(chunk: &[u8; 16], secret: &[u8], s: usize, seed: u64) -> u64 {
    if U128_KEY {
        let key = u128::from(r64(secret, s).wrapping_add(seed))
            | (u128::from(r64(secret, s + 8).wrapping_sub(seed)) << 64);
        let x = u128::from_le_bytes(*chunk) ^ key;
        mul128_fold64(x as u64, (x >> 64) as u64)
    } else {
        mix16(chunk, 0, secret, s, seed)
    }
}

// The mid-size paths iterate fixed-size chunks with a constant trip-count
// bound (`take`), so LLVM unrolls them, folds the secret offsets and drops the
// bounds checks. Indexed loops here were measured 25% slower (see CLAUDE.md).
#[inline(always)]
fn len_129to240_64<const U128_KEY: bool>(input: &[u8], secret: &ShortSecret, seed: u64) -> u64 {
    let len = input.len();
    let mut acc = (len as u64).wrapping_mul(PRIME64_1);
    let (head, tail) = input.split_at(128);
    let (head, _) = head.as_chunks::<16>();
    for (i, chunk) in head.iter().enumerate().take(8) {
        acc = acc.wrapping_add(mix16_chunk::<U128_KEY>(chunk, secret, 16 * i, seed));
    }
    acc = avalanche(acc);
    // len <= 240 leaves at most (240 - 128) / 16 = 7 whole chunks.
    let (tail, _) = tail.as_chunks::<16>();
    for (i, chunk) in tail.iter().enumerate().take(7) {
        acc = acc.wrapping_add(mix16_chunk::<U128_KEY>(
            chunk,
            secret,
            16 * i + MIDSIZE_STARTOFFSET,
            seed,
        ));
    }
    acc = acc.wrapping_add(mix16(
        input,
        len - 16,
        secret,
        XXH3_SECRET_SIZE_MIN - MIDSIZE_LASTOFFSET,
        seed,
    ));
    avalanche(acc)
}

#[inline(always)]
fn short_64(input: &[u8], secret: &ShortSecret, seed: u64) -> u64 {
    match input.len() {
        0..=16 => len_0to16_64(input, secret, seed),
        17..=128 => len_17to128_64(input, secret, seed),
        _ => len_129to240_64::<false>(input, secret, seed),
    }
}

// ---------------------------------------------------------------------------
// 128-bit short paths (len <= 240)

#[inline(always)]
fn to_u128(lo: u64, hi: u64) -> u128 {
    (u128::from(hi) << 64) | u128::from(lo)
}

#[inline(always)]
fn len_1to3_128(input: &[u8], secret: &ShortSecret, seed: u64) -> u128 {
    let len = input.len();
    let c1 = u32::from(input[0]);
    let c2 = u32::from(input[len >> 1]);
    let c3 = u32::from(input[len - 1]);
    let combinedl = (c1 << 16) | (c2 << 24) | c3 | ((len as u32) << 8);
    let combinedh = combinedl.swap_bytes().rotate_left(13);
    let bitflipl = u64::from(r32(secret, 0) ^ r32(secret, 4)).wrapping_add(seed);
    let bitfliph = u64::from(r32(secret, 8) ^ r32(secret, 12)).wrapping_sub(seed);
    to_u128(
        xxh64_avalanche(u64::from(combinedl) ^ bitflipl),
        xxh64_avalanche(u64::from(combinedh) ^ bitfliph),
    )
}

#[inline(always)]
fn len_4to8_128(input: &[u8], secret: &ShortSecret, mut seed: u64) -> u128 {
    let len = input.len();
    seed ^= u64::from((seed as u32).swap_bytes()) << 32;
    let input_lo = r32(input, 0);
    let input_hi = r32(input, len - 4);
    let input64 = u64::from(input_lo).wrapping_add(u64::from(input_hi) << 32);
    let bitflip = (r64(secret, 16) ^ r64(secret, 24)).wrapping_add(seed);
    let keyed = input64 ^ bitflip;
    let (mut lo, mut hi) = mult64to128(keyed, PRIME64_1.wrapping_add((len as u64) << 2));
    hi = hi.wrapping_add(lo << 1);
    lo ^= hi >> 3;
    lo = xorshift64(lo, 35);
    lo = lo.wrapping_mul(PRIME_MX2);
    lo = xorshift64(lo, 28);
    to_u128(lo, avalanche(hi))
}

#[inline(always)]
fn len_9to16_128(input: &[u8], secret: &ShortSecret, seed: u64) -> u128 {
    let len = input.len();
    let bitflipl = (r64(secret, 32) ^ r64(secret, 40)).wrapping_sub(seed);
    let bitfliph = (r64(secret, 48) ^ r64(secret, 56)).wrapping_add(seed);
    let input_lo = r64(input, 0);
    let mut input_hi = r64(input, len - 8);
    let (mut lo, mut hi) = mult64to128(input_lo ^ input_hi ^ bitflipl, PRIME64_1);
    lo = lo.wrapping_add(((len - 1) as u64) << 54);
    input_hi ^= bitfliph;
    hi = hi.wrapping_add(
        input_hi.wrapping_add(u64::from(input_hi as u32).wrapping_mul(u64::from(PRIME32_2 - 1))),
    );
    lo ^= hi.swap_bytes();
    let (h_lo, h_hi) = mult64to128(lo, PRIME64_2);
    let h_hi = h_hi.wrapping_add(hi.wrapping_mul(PRIME64_2));
    to_u128(avalanche(h_lo), avalanche(h_hi))
}

#[inline(always)]
fn len_0to16_128(input: &[u8], secret: &ShortSecret, seed: u64) -> u128 {
    match input.len() {
        9.. => len_9to16_128(input, secret, seed),
        4.. => len_4to8_128(input, secret, seed),
        1.. => len_1to3_128(input, secret, seed),
        0 => to_u128(
            xxh64_avalanche(seed ^ r64(secret, 64) ^ r64(secret, 72)),
            xxh64_avalanche(seed ^ r64(secret, 80) ^ r64(secret, 88)),
        ),
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn mix32b(
    acc: (u64, u64),
    input: &[u8],
    i1: usize,
    i2: usize,
    secret: &[u8],
    s: usize,
    seed: u64,
) -> (u64, u64) {
    let mut lo = acc.0.wrapping_add(mix16(input, i1, secret, s, seed));
    lo ^= r64(input, i2).wrapping_add(r64(input, i2 + 8));
    let mut hi = acc.1.wrapping_add(mix16(input, i2, secret, s + 16, seed));
    hi ^= r64(input, i1).wrapping_add(r64(input, i1 + 8));
    (lo, hi)
}

#[inline(always)]
fn finish_mid_128(acc: (u64, u64), len: usize, seed: u64) -> u128 {
    let lo = acc.0.wrapping_add(acc.1);
    let hi = acc
        .0
        .wrapping_mul(PRIME64_1)
        .wrapping_add(acc.1.wrapping_mul(PRIME64_4))
        .wrapping_add((len as u64).wrapping_sub(seed).wrapping_mul(PRIME64_2));
    to_u128(avalanche(lo), 0u64.wrapping_sub(avalanche(hi)))
}

#[inline(always)]
fn len_17to128_128(input: &[u8], secret: &ShortSecret, seed: u64) -> u128 {
    let len = input.len();
    let mut acc = ((len as u64).wrapping_mul(PRIME64_1), 0u64);
    if len > 32 {
        if len > 64 {
            if len > 96 {
                acc = mix32b(acc, input, 48, len - 64, secret, 96, seed);
            }
            acc = mix32b(acc, input, 32, len - 48, secret, 64, seed);
        }
        acc = mix32b(acc, input, 16, len - 32, secret, 32, seed);
    }
    acc = mix32b(acc, input, 0, len - 16, secret, 0, seed);
    finish_mid_128(acc, len, seed)
}

#[inline(always)]
fn len_129to240_128(input: &[u8], secret: &ShortSecret, seed: u64) -> u128 {
    let len = input.len();
    let mut acc = ((len as u64).wrapping_mul(PRIME64_1), 0u64);
    let (head, tail) = input.split_at(128);
    let (head, _) = head.as_chunks::<32>();
    for (i, chunk) in head.iter().enumerate().take(4) {
        acc = mix32b(acc, chunk, 0, 16, secret, 32 * i, seed);
    }
    acc = (avalanche(acc.0), avalanche(acc.1));
    // len <= 240 leaves at most (240 - 128) / 32 = 3 whole chunks.
    let (tail, _) = tail.as_chunks::<32>();
    for (i, chunk) in tail.iter().enumerate().take(3) {
        acc = mix32b(
            acc,
            chunk,
            0,
            16,
            secret,
            MIDSIZE_STARTOFFSET + 32 * i,
            seed,
        );
    }
    acc = mix32b(
        acc,
        input,
        len - 16,
        len - 32,
        secret,
        XXH3_SECRET_SIZE_MIN - MIDSIZE_LASTOFFSET - 16,
        0u64.wrapping_sub(seed),
    );
    finish_mid_128(acc, len, seed)
}

#[inline(always)]
fn short_128(input: &[u8], secret: &ShortSecret, seed: u64) -> u128 {
    match input.len() {
        0..=16 => len_0to16_128(input, secret, seed),
        17..=128 => len_17to128_128(input, secret, seed),
        _ => len_129to240_128(input, secret, seed),
    }
}

// ---------------------------------------------------------------------------
// Long inputs (len > 240)

pub(crate) const INIT_ACC: [u64; 8] = [
    PRIME32_3 as u64,
    PRIME64_1,
    PRIME64_2,
    PRIME64_3,
    PRIME64_4,
    PRIME32_2 as u64,
    PRIME64_5,
    PRIME32_1 as u64,
];

#[inline(always)]
fn merge_accs(acc: &[u64; 8], secret: &[u8], s: usize, start: u64) -> u64 {
    let mut result = start;
    for i in 0..4 {
        result = result.wrapping_add(mul128_fold64(
            acc[2 * i] ^ r64(secret, s + 16 * i),
            acc[2 * i + 1] ^ r64(secret, s + 16 * i + 8),
        ));
    }
    avalanche(result)
}

#[inline(always)]
pub(crate) fn finish_long_64(acc: &[u64; 8], secret: &[u8], len: u64) -> u64 {
    merge_accs(
        acc,
        secret,
        SECRET_MERGEACCS_START,
        len.wrapping_mul(PRIME64_1),
    )
}

#[inline(always)]
pub(crate) fn finish_long_128(acc: &[u64; 8], secret: &[u8], len: u64) -> u128 {
    let lo = merge_accs(
        acc,
        secret,
        SECRET_MERGEACCS_START,
        len.wrapping_mul(PRIME64_1),
    );
    let hi = merge_accs(
        acc,
        secret,
        secret.len() - STRIPE_LEN - SECRET_MERGEACCS_START,
        !len.wrapping_mul(PRIME64_2),
    );
    to_u128(lo, hi)
}

/// Runs the stripe accumulator over a whole one-shot input (`len > 240`).
#[inline(always)]
fn long_acc(input: &[u8], secret: &[u8]) -> [u64; 8] {
    let mut acc = [0; 8];
    // Every byte but the last goes through whole stripes; the final stripe is
    // the last 64 bytes, re-read with its own secret offset.
    let (stripes, _) = input[..input.len() - 1].as_chunks::<STRIPE_LEN>();
    let last = input.last_chunk::<STRIPE_LEN>();
    let mut so_far = 0;
    accumulate::consume(&mut acc, true, stripes, &mut so_far, secret, last);
    acc
}

// ---------------------------------------------------------------------------
// Public one-shot functions

// The entry points inline only the paths for inputs of at most 128 bytes.
// Inlining the 129..=240 or long paths into them adds register saves and a
// stack frame to every call, which cost 8- and 16-byte hashes 20-30% (see
// CLAUDE.md). The out-of-line helpers below still use the default secret as a
// constant, so its bytes fold into immediates.

// Seed 0 gets its own copy: with a constant seed, `secret + seed` folds into
// immediates. A runtime seed in the same body let LLVM's SLP vectorizer pack
// the (lo, hi) pairs into SSE registers (measured 18.2 ns vs 10.9 ns at 240 B).

#[inline(never)]
fn mid_64_default(data: &[u8]) -> u64 {
    len_129to240_64::<false>(data, default_short_secret(), 0)
}

#[inline(never)]
fn mid_64_seeded(data: &[u8], seed: u64) -> u64 {
    len_129to240_64::<true>(data, default_short_secret(), seed)
}

#[inline(never)]
fn mid_128_default(data: &[u8]) -> u128 {
    len_129to240_128(data, default_short_secret(), 0)
}

#[inline(never)]
fn mid_128_seeded(data: &[u8], seed: u64) -> u128 {
    len_129to240_128(data, default_short_secret(), seed)
}

#[inline(never)]
fn long_64_default(data: &[u8]) -> u64 {
    finish_long_64(
        &long_acc(data, &DEFAULT_SECRET),
        &DEFAULT_SECRET,
        data.len() as u64,
    )
}

#[inline(never)]
fn long_128_default(data: &[u8]) -> u128 {
    finish_long_128(
        &long_acc(data, &DEFAULT_SECRET),
        &DEFAULT_SECRET,
        data.len() as u64,
    )
}

#[inline(never)]
fn long_64_seeded(data: &[u8], seed: u64) -> u64 {
    let secret = derive_secret(seed);
    finish_long_64(&long_acc(data, &secret), &secret, data.len() as u64)
}

#[inline(never)]
fn long_128_seeded(data: &[u8], seed: u64) -> u128 {
    let secret = derive_secret(seed);
    finish_long_128(&long_acc(data, &secret), &secret, data.len() as u64)
}

/// Computes the 64-bit XXH3 hash of `data` (seed 0, default secret).
///
/// ```
/// assert_eq!(zenhash::xxh3_64(b""), 0x2D06_8005_38D3_94C2);
/// ```
#[must_use]
pub fn xxh3_64(data: &[u8]) -> u64 {
    match data.len() {
        0..=16 => len_0to16_64(data, default_short_secret(), 0),
        17..=128 => len_17to128_64(data, default_short_secret(), 0),
        129..=MID_SIZE_MAX => mid_64_default(data),
        _ => long_64_default(data),
    }
}

/// Computes the 64-bit XXH3 hash of `data` with `seed`. Seed 0 gives the
/// same value as [`xxh3_64`].
///
/// ```
/// assert_eq!(zenhash::xxh3_64_with_seed(b"abc", 0), zenhash::xxh3_64(b"abc"));
/// assert_ne!(zenhash::xxh3_64_with_seed(b"abc", 1), zenhash::xxh3_64(b"abc"));
/// ```
#[must_use]
pub fn xxh3_64_with_seed(data: &[u8], seed: u64) -> u64 {
    match data.len() {
        0..=16 => len_0to16_64(data, default_short_secret(), seed),
        17..=128 => len_17to128_64(data, default_short_secret(), seed),
        129..=MID_SIZE_MAX if seed == 0 => mid_64_default(data),
        129..=MID_SIZE_MAX => mid_64_seeded(data, seed),
        _ if seed == 0 => long_64_default(data),
        _ => long_64_seeded(data, seed),
    }
}

/// Computes the 64-bit XXH3 hash of `data` with a custom `secret` of at least
/// [`XXH3_SECRET_SIZE_MIN`] bytes.
///
/// The secret should look random: the xxHash documentation recommends
/// generating one from a high-entropy source. Returns
/// [`Error::SecretTooShort`] for a shorter secret.
///
/// ```
/// let secret = [0x5Au8; 136]; // use high-entropy bytes in practice
/// let h = zenhash::xxh3_64_with_secret(b"data", &secret)?;
/// assert_ne!(h, zenhash::xxh3_64(b"data"));
/// # Ok::<(), whereat::At<zenhash::Error>>(())
/// ```
pub fn xxh3_64_with_secret(data: &[u8], secret: &[u8]) -> Result<u64, At<Error>> {
    let short = validate_secret(secret)?;
    Ok(if data.len() <= MID_SIZE_MAX {
        short_64(data, short, 0)
    } else {
        finish_long_64(&long_acc(data, secret), secret, data.len() as u64)
    })
}

/// Computes the 128-bit XXH3 hash of `data` (seed 0, default secret).
///
/// ```
/// assert_eq!(zenhash::xxh3_128(b""), 0x99AA_06D3_0147_98D8_6001_C324_468D_497F);
/// ```
#[must_use]
pub fn xxh3_128(data: &[u8]) -> u128 {
    match data.len() {
        0..=16 => len_0to16_128(data, default_short_secret(), 0),
        17..=128 => len_17to128_128(data, default_short_secret(), 0),
        129..=MID_SIZE_MAX => mid_128_default(data),
        _ => long_128_default(data),
    }
}

/// Computes the 128-bit XXH3 hash of `data` with `seed`. Seed 0 gives the
/// same value as [`xxh3_128`]. Do not truncate it in place of
/// [`xxh3_64_with_seed`]: the low 64 bits match that function only for
/// inputs of 1 to 3 bytes or over 240 bytes.
#[must_use]
pub fn xxh3_128_with_seed(data: &[u8], seed: u64) -> u128 {
    match data.len() {
        0..=16 => len_0to16_128(data, default_short_secret(), seed),
        17..=128 => len_17to128_128(data, default_short_secret(), seed),
        129..=MID_SIZE_MAX if seed == 0 => mid_128_default(data),
        129..=MID_SIZE_MAX => mid_128_seeded(data, seed),
        _ if seed == 0 => long_128_default(data),
        _ => long_128_seeded(data, seed),
    }
}

/// Computes the 128-bit XXH3 hash of `data` with a custom `secret` of at least
/// [`XXH3_SECRET_SIZE_MIN`] bytes. Returns [`Error::SecretTooShort`] for a
/// shorter secret.
pub fn xxh3_128_with_secret(data: &[u8], secret: &[u8]) -> Result<u128, At<Error>> {
    let short = validate_secret(secret)?;
    Ok(if data.len() <= MID_SIZE_MAX {
        short_128(data, short, 0)
    } else {
        finish_long_128(&long_acc(data, secret), secret, data.len() as u64)
    })
}

/// Where a streaming hasher's secret lives.
#[derive(Clone, Debug)]
pub(crate) enum SecretStore {
    /// The default secret, seed 0.
    Default,
    /// A non-zero seed: short inputs use the default secret plus the seed,
    /// long inputs the derived secret.
    Seeded(u64, Box<[u8; SECRET_DEFAULT_SIZE]>),
    /// A validated custom secret.
    Custom(Box<[u8]>),
}

impl SecretStore {
    #[inline(always)]
    pub(crate) fn long_secret(&self) -> &[u8] {
        match self {
            SecretStore::Default => &DEFAULT_SECRET,
            SecretStore::Seeded(_, s) => &s[..],
            SecretStore::Custom(s) => s,
        }
    }

    pub(crate) fn short_64(&self, input: &[u8]) -> u64 {
        match self {
            SecretStore::Default => short_64(input, default_short_secret(), 0),
            SecretStore::Seeded(seed, _) => short_64(input, default_short_secret(), *seed),
            SecretStore::Custom(s) => short_64(input, short_secret(s).unwrap_or(&[0; 136]), 0),
        }
    }

    pub(crate) fn short_128(&self, input: &[u8]) -> u128 {
        match self {
            SecretStore::Default => short_128(input, default_short_secret(), 0),
            SecretStore::Seeded(seed, _) => short_128(input, default_short_secret(), *seed),
            SecretStore::Custom(s) => short_128(input, short_secret(s).unwrap_or(&[0; 136]), 0),
        }
    }
}
