//! The XXH3 stripe accumulator: the only part of XXH3 that runs SIMD.
//!
//! Each tier keeps the eight 64-bit accumulators in registers for the whole
//! call. [`consume`] dispatches once per call: per hash for one-shot inputs,
//! per `update` (at most every 256 bytes of buffered input, or once per large
//! slice) when streaming.
//!
//! magetypes 0.9.29 has no 32x32->64 widening multiply or 64-bit lane swap on
//! its `u64` vectors, which XXH3 needs in every stripe, so each tier is written
//! with the intrinsics of its instruction set through `#[arcane]`/`#[rite]`.
//! The tiers share one loop, [`consume_body!`], and differ only in four
//! helpers: load, store, accumulate one stripe, scramble.

#[allow(unused_imports)]
use archmage::prelude::*;

use super::{INIT_ACC, SECRET_CONSUME_RATE, SECRET_LASTACC_START, STRIPE_LEN};

/// The 64 secret bytes at `off`. Callers guarantee `off + 64 <= secret.len()`.
#[inline(always)]
fn key_at(secret: &[u8], off: usize) -> &[u8; 64] {
    match secret[off..].first_chunk::<64>() {
        Some(k) => k,
        // Unreachable: every caller's offset leaves at least 64 bytes.
        None => &[0; 64],
    }
}

/// Accumulates `stripes` into `acc` (or, when `fresh`, into the initial
/// accumulators; `acc` is then output only), scrambling at every block boundary. Then,
/// if `last` is given, accumulates it with the last-stripe secret offset (no
/// scramble). `so_far` counts stripes already accumulated in the current block.
///
/// `secret` must be at least 136 bytes long, which the public API validates.
pub(super) fn consume(
    acc: &mut [u64; 8],
    fresh: bool,
    stripes: &[[u8; STRIPE_LEN]],
    so_far: &mut usize,
    secret: &[u8],
    last: Option<&[u8; STRIPE_LEN]>,
) {
    // The SIMD tiers read lanes in native byte order.
    if cfg!(target_endian = "big") {
        return consume_scalar(ScalarToken, acc, fresh, stripes, so_far, secret, last);
    }
    incant!(
        consume(acc, fresh, stripes, so_far, secret, last),
        [v4(cfg(avx512)), v3, v1, neon, wasm128, scalar]
    )
}

/// Stripes per block under the default secret: (192 - 64) / 8.
const DEFAULT_BLOCK_STRIPES: usize = 16;

/// The secret as a fixed-size array when its blocks are 16 stripes long.
#[inline(always)]
fn default_block_keys(secret: &[u8], per_block: usize) -> Option<&[u8; 192]> {
    if per_block == DEFAULT_BLOCK_STRIPES {
        secret.first_chunk::<192>()
    } else {
        None
    }
}

/// The block loop every tier shares. `$load`/`$store` move the accumulators
/// between `[u64; 8]` and the tier's registers; `$accumulate` and `$scramble`
/// take and return that register state.
macro_rules! consume_body {
    ($acc:ident, $fresh:ident, $stripes:ident, $so_far:ident, $secret:ident, $last:ident,
     $load:ident, $store:ident, $accumulate:ident, $scramble:ident) => {{
        let per_block = ($secret.len() - STRIPE_LEN) / SECRET_CONSUME_RATE;
        let scramble_key = key_at($secret, $secret.len() - STRIPE_LEN);
        // A fresh hash loads the initial accumulators from the read-only
        // constant. Loading them as one wide vector from a stack copy that
        // was just written in narrower pieces defeats store forwarding: it
        // cost 241..1024-byte AVX-512 hashes up to 11 ns (see CLAUDE.md).
        let mut state = if $fresh {
            $load(&INIT_ACC)
        } else {
            $load($acc)
        };
        let mut stripes = $stripes;
        while !stripes.is_empty() {
            // Fast path: a block that starts at stripe 0 under a 192..=199-byte
            // secret (the default and every seed-derived secret). Covers the
            // partial last block every one-shot hash ends with. Constant-bounded
            // key offsets remove the per-stripe bookkeeping and bounds checks.
            if *$so_far == 0 {
                if let Some(keys) = default_block_keys($secret, per_block) {
                    let k = stripes.len().min(DEFAULT_BLOCK_STRIPES);
                    let (block, rest) = stripes.split_at(k);
                    // Two stripes per iteration halves the loop overhead.
                    let (pairs, odd) = block.as_chunks::<2>();
                    for (i, [a, b]) in pairs.iter().enumerate().take(DEFAULT_BLOCK_STRIPES / 2) {
                        let off = 2 * i * SECRET_CONSUME_RATE;
                        state = $accumulate(state, a, key_at(keys, off));
                        state = $accumulate(state, b, key_at(keys, off + SECRET_CONSUME_RATE));
                    }
                    if let [c] = odd {
                        // An odd stripe exists only when k < 16; the mask is a
                        // no-op that proves the offset in bounds.
                        let off =
                            (2 * pairs.len() & (DEFAULT_BLOCK_STRIPES - 1)) * SECRET_CONSUME_RATE;
                        state = $accumulate(state, c, key_at(keys, off));
                    }
                    if k == DEFAULT_BLOCK_STRIPES {
                        state = $scramble(state, scramble_key);
                    } else {
                        *$so_far = k;
                    }
                    stripes = rest;
                    continue;
                }
            }
            let n = (per_block - *$so_far).min(stripes.len());
            let (now, rest) = stripes.split_at(n);
            if let Some(keys) = default_block_keys($secret, per_block) {
                // Partial block under a 192-byte secret (streaming leaves
                // these at every update). `k < 16` always holds; the mask
                // lets LLVM prove `k * 8 + 64 <= 192` without a check.
                for (k, stripe) in (*$so_far..).zip(now) {
                    let off = (k & (DEFAULT_BLOCK_STRIPES - 1)) * SECRET_CONSUME_RATE;
                    state = $accumulate(state, stripe, key_at(keys, off));
                }
            } else {
                // Windows of exactly 64 bytes let LLVM drop the per-stripe
                // bounds check that indexing needs. `first_chunk` on a
                // window never fails, so the `else` is unreachable.
                let keys = $secret[*$so_far * SECRET_CONSUME_RATE..]
                    .windows(STRIPE_LEN)
                    .step_by(SECRET_CONSUME_RATE);
                for (stripe, key) in now.iter().zip(keys) {
                    let Some(key) = key.first_chunk::<STRIPE_LEN>() else {
                        break;
                    };
                    state = $accumulate(state, stripe, key);
                }
            }
            *$so_far += n;
            if *$so_far == per_block {
                state = $scramble(state, scramble_key);
                *$so_far = 0;
            }
            stripes = rest;
        }
        if let Some(last) = $last {
            let off = $secret.len() - STRIPE_LEN - SECRET_LASTACC_START;
            state = $accumulate(state, last, key_at($secret, off));
        }
        $store(state, $acc);
    }};
}

// ---------------------------------------------------------------------------
// Scalar: the specification's reference formulation.

#[inline(always)]
fn load_scalar(acc: &[u64; 8]) -> [u64; 8] {
    *acc
}

#[inline(always)]
fn store_scalar(state: [u64; 8], acc: &mut [u64; 8]) {
    *acc = state;
}

#[inline(always)]
fn accumulate_scalar(mut acc: [u64; 8], stripe: &[u8; 64], key: &[u8; 64]) -> [u64; 8] {
    let (data, _) = stripe.as_chunks::<8>();
    let (keys, _) = key.as_chunks::<8>();
    for i in 0..8 {
        let data_val = u64::from_le_bytes(data[i]);
        let data_key = data_val ^ u64::from_le_bytes(keys[i]);
        acc[i ^ 1] = acc[i ^ 1].wrapping_add(data_val);
        acc[i] = acc[i].wrapping_add((data_key & 0xFFFF_FFFF).wrapping_mul(data_key >> 32));
    }
    acc
}

#[inline(always)]
fn scramble_scalar(mut acc: [u64; 8], key: &[u8; 64]) -> [u64; 8] {
    let (keys, _) = key.as_chunks::<8>();
    for (a, k) in acc.iter_mut().zip(keys) {
        let mut v = *a;
        v ^= v >> 47;
        v ^= u64::from_le_bytes(*k);
        *a = v.wrapping_mul(u64::from(crate::common::PRIME32_1));
    }
    acc
}

// Out of line for the same reason as `consume_v1`.
#[inline(never)]
fn consume_scalar(
    _token: ScalarToken,
    acc: &mut [u64; 8],
    fresh: bool,
    stripes: &[[u8; STRIPE_LEN]],
    so_far: &mut usize,
    secret: &[u8],
    last: Option<&[u8; STRIPE_LEN]>,
) {
    consume_body!(
        acc,
        fresh,
        stripes,
        so_far,
        secret,
        last,
        load_scalar,
        store_scalar,
        accumulate_scalar,
        scramble_scalar
    )
}

// ---------------------------------------------------------------------------
// x86-64

#[cfg(target_arch = "x86_64")]
mod x86 {
    use archmage::prelude::*;

    use super::{
        DEFAULT_BLOCK_STRIPES, INIT_ACC, SECRET_CONSUME_RATE, SECRET_LASTACC_START, STRIPE_LEN,
        default_block_keys, key_at,
    };
    use crate::common::PRIME32_1;

    // The high 32 bits of each 64-bit lane reach the low half for `mul_epu32`
    // through a shift, not `shuffle_epi32` as in the C reference: the shift
    // runs on a different port than the data-swap shuffle (see CLAUDE.md).
    // _MM_SHUFFLE(1, 0, 3, 2): swaps the two 64-bit halves of each 128-bit lane.
    const SWAP64: i32 = 0b01_00_11_10;

    // --- SSE2 (v1): four 128-bit registers -------------------------------

    #[rite(v1, import_intrinsics)]
    fn load_v1(acc: &[u64; 8]) -> [__m128i; 4] {
        let (a, _) = acc.as_chunks::<2>();
        [
            _mm_loadu_si128(&a[0]),
            _mm_loadu_si128(&a[1]),
            _mm_loadu_si128(&a[2]),
            _mm_loadu_si128(&a[3]),
        ]
    }

    #[rite(v1, import_intrinsics)]
    fn store_v1(state: [__m128i; 4], acc: &mut [u64; 8]) {
        let (a, _) = acc.as_chunks_mut::<2>();
        for (dst, v) in a.iter_mut().zip(state) {
            _mm_storeu_si128(dst, v);
        }
    }

    #[rite(v1, import_intrinsics)]
    fn accumulate_v1(mut acc: [__m128i; 4], stripe: &[u8; 64], key: &[u8; 64]) -> [__m128i; 4] {
        let (data, _) = stripe.as_chunks::<16>();
        let (keys, _) = key.as_chunks::<16>();
        for i in 0..4 {
            let data_vec = _mm_loadu_si128(&data[i]);
            let data_key = _mm_xor_si128(data_vec, _mm_loadu_si128(&keys[i]));
            let data_key_lo = _mm_srli_epi64::<32>(data_key);
            let product = _mm_mul_epu32(data_key, data_key_lo);
            let data_swap = _mm_shuffle_epi32::<SWAP64>(data_vec);
            acc[i] = _mm_add_epi64(product, _mm_add_epi64(acc[i], data_swap));
        }
        acc
    }

    #[rite(v1, import_intrinsics)]
    fn scramble_v1(mut acc: [__m128i; 4], key: &[u8; 64]) -> [__m128i; 4] {
        let (keys, _) = key.as_chunks::<16>();
        let prime = _mm_set1_epi32(PRIME32_1 as i32);
        for i in 0..4 {
            let shifted = _mm_xor_si128(acc[i], _mm_srli_epi64::<47>(acc[i]));
            let data_key = _mm_xor_si128(shifted, _mm_loadu_si128(&keys[i]));
            let data_key_hi = _mm_srli_epi64::<32>(data_key);
            let prod_lo = _mm_mul_epu32(data_key, prime);
            let prod_hi = _mm_mul_epu32(data_key_hi, prime);
            acc[i] = _mm_add_epi64(prod_lo, _mm_slli_epi64::<32>(prod_hi));
        }
        acc
    }

    /// SSE2 is the x86-64 baseline, so `incant!` reaches this tier without a
    /// CPU check. Out of line, it keeps its register saves out of the
    /// dispatcher's path to the AVX2 kernel (see CLAUDE.md).
    #[inline(never)]
    pub(super) fn consume_v1(
        token: X64V1Token,
        acc: &mut [u64; 8],
        fresh: bool,
        stripes: &[[u8; STRIPE_LEN]],
        so_far: &mut usize,
        secret: &[u8],
        last: Option<&[u8; STRIPE_LEN]>,
    ) {
        consume_v1_kernel(token, acc, fresh, stripes, so_far, secret, last);
    }

    #[arcane(import_intrinsics)]
    fn consume_v1_kernel(
        _token: X64V1Token,
        acc: &mut [u64; 8],
        fresh: bool,
        stripes: &[[u8; STRIPE_LEN]],
        so_far: &mut usize,
        secret: &[u8],
        last: Option<&[u8; STRIPE_LEN]>,
    ) {
        consume_body!(
            acc,
            fresh,
            stripes,
            so_far,
            secret,
            last,
            load_v1,
            store_v1,
            accumulate_v1,
            scramble_v1
        )
    }

    // --- AVX2 (v3): two 256-bit registers --------------------------------

    #[rite(v3, import_intrinsics)]
    fn load_v3(acc: &[u64; 8]) -> [__m256i; 2] {
        let (a, _) = acc.as_chunks::<4>();
        [_mm256_loadu_si256(&a[0]), _mm256_loadu_si256(&a[1])]
    }

    #[rite(v3, import_intrinsics)]
    fn store_v3(state: [__m256i; 2], acc: &mut [u64; 8]) {
        let (a, _) = acc.as_chunks_mut::<4>();
        for (dst, v) in a.iter_mut().zip(state) {
            _mm256_storeu_si256(dst, v);
        }
    }

    #[rite(v3, import_intrinsics)]
    fn accumulate_v3(mut acc: [__m256i; 2], stripe: &[u8; 64], key: &[u8; 64]) -> [__m256i; 2] {
        let (data, _) = stripe.as_chunks::<32>();
        let (keys, _) = key.as_chunks::<32>();
        for i in 0..2 {
            let data_vec = _mm256_loadu_si256(&data[i]);
            let data_key = _mm256_xor_si256(data_vec, _mm256_loadu_si256(&keys[i]));
            let data_key_lo = _mm256_srli_epi64::<32>(data_key);
            let product = _mm256_mul_epu32(data_key, data_key_lo);
            let data_swap = _mm256_shuffle_epi32::<SWAP64>(data_vec);
            acc[i] = _mm256_add_epi64(product, _mm256_add_epi64(acc[i], data_swap));
        }
        acc
    }

    #[rite(v3, import_intrinsics)]
    fn scramble_v3(mut acc: [__m256i; 2], key: &[u8; 64]) -> [__m256i; 2] {
        let (keys, _) = key.as_chunks::<32>();
        let prime = _mm256_set1_epi32(PRIME32_1 as i32);
        for i in 0..2 {
            let shifted = _mm256_xor_si256(acc[i], _mm256_srli_epi64::<47>(acc[i]));
            let data_key = _mm256_xor_si256(shifted, _mm256_loadu_si256(&keys[i]));
            let data_key_hi = _mm256_srli_epi64::<32>(data_key);
            let prod_lo = _mm256_mul_epu32(data_key, prime);
            let prod_hi = _mm256_mul_epu32(data_key_hi, prime);
            acc[i] = _mm256_add_epi64(prod_lo, _mm256_slli_epi64::<32>(prod_hi));
        }
        acc
    }

    #[arcane(import_intrinsics)]
    pub(super) fn consume_v3(
        _token: X64V3Token,
        acc: &mut [u64; 8],
        fresh: bool,
        stripes: &[[u8; STRIPE_LEN]],
        so_far: &mut usize,
        secret: &[u8],
        last: Option<&[u8; STRIPE_LEN]>,
    ) {
        consume_body!(
            acc,
            fresh,
            stripes,
            so_far,
            secret,
            last,
            load_v3,
            store_v3,
            accumulate_v3,
            scramble_v3
        )
    }

    // --- AVX-512 (v4): one 512-bit register ------------------------------

    #[cfg(feature = "avx512")]
    #[rite(v4, import_intrinsics)]
    fn load_v4(acc: &[u64; 8]) -> __m512i {
        _mm512_loadu_si512(acc)
    }

    #[cfg(feature = "avx512")]
    #[rite(v4, import_intrinsics)]
    fn store_v4(state: __m512i, acc: &mut [u64; 8]) {
        _mm512_storeu_si512(acc, state);
    }

    #[cfg(feature = "avx512")]
    #[rite(v4, import_intrinsics)]
    fn accumulate_v4(acc: __m512i, stripe: &[u8; 64], key: &[u8; 64]) -> __m512i {
        let data_vec = _mm512_loadu_si512(stripe);
        let data_key = _mm512_xor_si512(data_vec, _mm512_loadu_si512(key));
        let data_key_lo = _mm512_srli_epi64::<32>(data_key);
        let product = _mm512_mul_epu32(data_key, data_key_lo);
        let data_swap = _mm512_shuffle_epi32::<SWAP64>(data_vec);
        _mm512_add_epi64(product, _mm512_add_epi64(acc, data_swap))
    }

    #[cfg(feature = "avx512")]
    #[rite(v4, import_intrinsics)]
    fn scramble_v4(acc: __m512i, key: &[u8; 64]) -> __m512i {
        let prime = _mm512_set1_epi32(PRIME32_1 as i32);
        // 0x96 = a ^ b ^ c: one instruction for acc ^ (acc >> 47) ^ key.
        let data_key = _mm512_ternarylogic_epi32::<0x96>(
            acc,
            _mm512_srli_epi64::<47>(acc),
            _mm512_loadu_si512(key),
        );
        let data_key_hi = _mm512_srli_epi64::<32>(data_key);
        let prod_lo = _mm512_mul_epu32(data_key, prime);
        let prod_hi = _mm512_mul_epu32(data_key_hi, prime);
        _mm512_add_epi64(prod_lo, _mm512_slli_epi64::<32>(prod_hi))
    }

    #[cfg(feature = "avx512")]
    #[arcane(import_intrinsics)]
    pub(super) fn consume_v4(
        _token: X64V4Token,
        acc: &mut [u64; 8],
        fresh: bool,
        stripes: &[[u8; STRIPE_LEN]],
        so_far: &mut usize,
        secret: &[u8],
        last: Option<&[u8; STRIPE_LEN]>,
    ) {
        consume_body!(
            acc,
            fresh,
            stripes,
            so_far,
            secret,
            last,
            load_v4,
            store_v4,
            accumulate_v4,
            scramble_v4
        )
    }
}

#[cfg(target_arch = "x86_64")]
use x86::*;

// ---------------------------------------------------------------------------
// AArch64 NEON: four 128-bit registers

#[cfg(target_arch = "aarch64")]
mod arm {
    use archmage::prelude::*;

    use super::{
        DEFAULT_BLOCK_STRIPES, INIT_ACC, SECRET_CONSUME_RATE, SECRET_LASTACC_START, STRIPE_LEN,
        default_block_keys, key_at,
    };
    use crate::common::PRIME32_1;

    #[rite(neon, import_intrinsics)]
    fn load_neon(acc: &[u64; 8]) -> [uint64x2_t; 4] {
        let (a, _) = acc.as_chunks::<2>();
        [
            vld1q_u64(&a[0]),
            vld1q_u64(&a[1]),
            vld1q_u64(&a[2]),
            vld1q_u64(&a[3]),
        ]
    }

    #[rite(neon, import_intrinsics)]
    fn store_neon(state: [uint64x2_t; 4], acc: &mut [u64; 8]) {
        let (a, _) = acc.as_chunks_mut::<2>();
        for (dst, v) in a.iter_mut().zip(state) {
            vst1q_u64(dst, v);
        }
    }

    #[rite(neon, import_intrinsics)]
    fn accumulate_neon(
        mut acc: [uint64x2_t; 4],
        stripe: &[u8; 64],
        key: &[u8; 64],
    ) -> [uint64x2_t; 4] {
        let (data, _) = stripe.as_chunks::<16>();
        let (keys, _) = key.as_chunks::<16>();
        for i in 0..4 {
            let data_vec = vreinterpretq_u64_u8(vld1q_u8(&data[i]));
            let key_vec = vreinterpretq_u64_u8(vld1q_u8(&keys[i]));
            let data_key = veorq_u64(data_vec, key_vec);
            let data_swap = vextq_u64::<1>(data_vec, data_vec);
            let lo = vmovn_u64(data_key);
            let hi = vshrn_n_u64::<32>(data_key);
            acc[i] = vmlal_u32(vaddq_u64(acc[i], data_swap), lo, hi);
        }
        acc
    }

    #[rite(neon, import_intrinsics)]
    fn scramble_neon(mut acc: [uint64x2_t; 4], key: &[u8; 64]) -> [uint64x2_t; 4] {
        let (keys, _) = key.as_chunks::<16>();
        let prime = vdup_n_u32(PRIME32_1);
        for i in 0..4 {
            let shifted = veorq_u64(acc[i], vshrq_n_u64::<47>(acc[i]));
            let data_key = veorq_u64(shifted, vreinterpretq_u64_u8(vld1q_u8(&keys[i])));
            let lo = vmovn_u64(data_key);
            let hi = vshrn_n_u64::<32>(data_key);
            let prod_hi = vshlq_n_u64::<32>(vmull_u32(hi, prime));
            acc[i] = vmlal_u32(prod_hi, lo, prime);
        }
        acc
    }

    #[arcane(import_intrinsics)]
    pub(super) fn consume_neon(
        _token: NeonToken,
        acc: &mut [u64; 8],
        fresh: bool,
        stripes: &[[u8; STRIPE_LEN]],
        so_far: &mut usize,
        secret: &[u8],
        last: Option<&[u8; STRIPE_LEN]>,
    ) {
        consume_body!(
            acc,
            fresh,
            stripes,
            so_far,
            secret,
            last,
            load_neon,
            store_neon,
            accumulate_neon,
            scramble_neon
        )
    }
}

#[cfg(target_arch = "aarch64")]
use arm::*;

// ---------------------------------------------------------------------------
// WASM SIMD128: four 128-bit registers

#[cfg(target_arch = "wasm32")]
mod wasm {
    use archmage::prelude::*;

    use super::{
        DEFAULT_BLOCK_STRIPES, INIT_ACC, SECRET_CONSUME_RATE, SECRET_LASTACC_START, STRIPE_LEN,
        default_block_keys, key_at,
    };
    use crate::common::PRIME32_1;

    #[rite(wasm128, import_intrinsics)]
    fn load_wasm128(acc: &[u64; 8]) -> [v128; 4] {
        let (a, _) = acc.as_chunks::<2>();
        [
            v128_load(&a[0]),
            v128_load(&a[1]),
            v128_load(&a[2]),
            v128_load(&a[3]),
        ]
    }

    #[rite(wasm128, import_intrinsics)]
    fn store_wasm128(state: [v128; 4], acc: &mut [u64; 8]) {
        let (a, _) = acc.as_chunks_mut::<2>();
        for (dst, v) in a.iter_mut().zip(state) {
            v128_store(dst, v);
        }
    }

    #[rite(wasm128, import_intrinsics)]
    fn accumulate_wasm128(mut acc: [v128; 4], stripe: &[u8; 64], key: &[u8; 64]) -> [v128; 4] {
        let (data, _) = stripe.as_chunks::<16>();
        let (keys, _) = key.as_chunks::<16>();
        for i in 0..4 {
            let data_vec = v128_load(&data[i]);
            let data_key = v128_xor(data_vec, v128_load(&keys[i]));
            // Low and high 32-bit halves of each 64-bit lane, packed into the
            // low two 32-bit lanes for the widening multiply.
            let lo = i32x4_shuffle::<0, 2, 0, 2>(data_key, data_key);
            let hi = i32x4_shuffle::<1, 3, 1, 3>(data_key, data_key);
            let product = u64x2_extmul_low_u32x4(lo, hi);
            let data_swap = i64x2_shuffle::<1, 0>(data_vec, data_vec);
            acc[i] = i64x2_add(product, i64x2_add(acc[i], data_swap));
        }
        acc
    }

    #[rite(wasm128, import_intrinsics)]
    fn scramble_wasm128(mut acc: [v128; 4], key: &[u8; 64]) -> [v128; 4] {
        let (keys, _) = key.as_chunks::<16>();
        let prime = u64x2_splat(u64::from(PRIME32_1));
        for i in 0..4 {
            let shifted = v128_xor(acc[i], u64x2_shr(acc[i], 47));
            let data_key = v128_xor(shifted, v128_load(&keys[i]));
            acc[i] = i64x2_mul(data_key, prime);
        }
        acc
    }

    #[arcane(import_intrinsics)]
    pub(super) fn consume_wasm128(
        _token: Wasm128Token,
        acc: &mut [u64; 8],
        fresh: bool,
        stripes: &[[u8; STRIPE_LEN]],
        so_far: &mut usize,
        secret: &[u8],
        last: Option<&[u8; STRIPE_LEN]>,
    ) {
        consume_body!(
            acc,
            fresh,
            stripes,
            so_far,
            secret,
            last,
            load_wasm128,
            store_wasm128,
            accumulate_wasm128,
            scramble_wasm128
        )
    }
}

#[cfg(target_arch = "wasm32")]
use wasm::*;
