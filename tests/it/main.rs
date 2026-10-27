#![cfg(any(
    feature = "xxh3",
    feature = "xxh32",
    feature = "xxh64",
    feature = "rapidhash",
    feature = "fnv"
))]

// One integration-test binary: every top-level file in tests/ links
// separately, so new test files go here as modules.
#[cfg(any(feature = "xxh3", feature = "xxh32", feature = "xxh64"))]
mod api;
#[cfg(feature = "xxh3")]
mod fuzz_regression;
#[cfg(any(feature = "xxh3", feature = "xxh32", feature = "xxh64"))]
mod oracle;
#[cfg(any(feature = "xxh3", feature = "xxh32", feature = "xxh64"))]
mod random;
#[cfg(any(feature = "xxh3", feature = "xxh32", feature = "xxh64"))]
mod vectors;

#[cfg(feature = "rapidhash")]
mod rapid;

#[cfg(all(
    feature = "xxh3",
    feature = "xxh32",
    feature = "xxh64",
    feature = "rapidhash"
))]
mod conformance;

/// Deterministic test bytes (splitmix64), so failures reproduce.
pub(crate) fn test_bytes(len: usize, seed: u64) -> Vec<u8> {
    let mut state = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut out = Vec::with_capacity(len + 8);
    while out.len() < len {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        out.extend_from_slice(&(z ^ (z >> 31)).to_le_bytes());
    }
    out.truncate(len);
    out
}

/// Lengths that hit every XXH3 path boundary plus block and buffer edges.
#[cfg(any(feature = "xxh3", feature = "xxh32", feature = "xxh64"))]
pub(crate) fn interesting_lengths() -> Vec<usize> {
    let mut v: Vec<usize> = (0..=300).collect();
    for base in [
        511, 512, 513, 1023, 1024, 1025, 1088, 2047, 2048, 2049, 4095, 4096, 4097, 8191, 8192,
        10_000, 16_384, 65_536, 100_003,
    ] {
        v.push(base);
    }
    v
}

#[cfg(all(
    feature = "xxh3",
    feature = "xxh32",
    feature = "xxh64",
    feature = "rapidhash"
))]
#[test]
fn c_reference_conformance_matrix() {
    conformance::verify();
}

#[path = "../support/stability.rs"]
mod stability;
#[cfg(any(
    feature = "xxh3",
    feature = "xxh32",
    feature = "xxh64",
    feature = "rapidhash"
))]
#[test]
fn expanded_stability() {
    stability::verify();
}
#[cfg(feature = "fnv")]
#[path = "../support/fnv.rs"]
mod fnv;
