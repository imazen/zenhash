//! Constants and little-endian readers shared by every algorithm.

pub(crate) const PRIME32_1: u32 = 0x9E37_79B1;
pub(crate) const PRIME32_2: u32 = 0x85EB_CA77;
pub(crate) const PRIME32_3: u32 = 0xC2B2_AE3D;
pub(crate) const PRIME32_4: u32 = 0x27D4_EB2F;
pub(crate) const PRIME32_5: u32 = 0x1656_67B1;

pub(crate) const PRIME64_1: u64 = 0x9E37_79B1_85EB_CA87;
pub(crate) const PRIME64_2: u64 = 0xC2B2_AE3D_27D4_EB4F;
pub(crate) const PRIME64_3: u64 = 0x1656_67B1_9E37_79F9;
pub(crate) const PRIME64_4: u64 = 0x85EB_CA77_C2B2_AE63;
pub(crate) const PRIME64_5: u64 = 0x27D4_EB2F_1656_67C5;

/// Reads a little-endian `u32` at `i`. Callers guarantee `i + 4 <= b.len()`.
#[inline(always)]
pub(crate) fn r32(b: &[u8], i: usize) -> u32 {
    let mut a = [0u8; 4];
    a.copy_from_slice(&b[i..i + 4]);
    u32::from_le_bytes(a)
}

/// Reads a little-endian `u64` at `i`. Callers guarantee `i + 8 <= b.len()`.
#[inline(always)]
pub(crate) fn r64(b: &[u8], i: usize) -> u64 {
    let mut a = [0u8; 8];
    a.copy_from_slice(&b[i..i + 8]);
    u64::from_le_bytes(a)
}

/// Fills `buf[*len..]` from the front of `input` and returns the unused rest of
/// `input`. Shared by the streaming hashers.
#[inline(always)]
pub(crate) fn fill<'a>(buf: &mut [u8], len: &mut usize, input: &'a [u8]) -> &'a [u8] {
    let n = (buf.len() - *len).min(input.len());
    let (head, rest) = input.split_at(n);
    buf[*len..*len + n].copy_from_slice(head);
    *len += n;
    rest
}
