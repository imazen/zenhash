//! Every SIMD tier the CPU can run must give the reference answer.
//!
//! Its own test binary on purpose (an exception to the one-binary rule in
//! tests/it): `for_each_token_permutation` disables tiers process-wide. Inside
//! the shared binary, tests running in parallel randomly landed on reduced
//! tiers, and a broken scalar kernel slipped past this test when the suite
//! ran in parallel (found by cargo-mutants; see CLAUDE.md). A separate
//! process makes every permutation deterministic.

use archmage::testing::{CompileTimePolicy, for_each_token_permutation};

/// Deterministic test bytes (splitmix64).
fn test_bytes(len: usize, seed: u64) -> Vec<u8> {
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

struct Case {
    data: Vec<u8>,
    h64: u64,
    h128: u128,
    seeded64: u64,
    seeded128: u128,
    secret64: u64,
    secret128: u128,
}

#[test]
fn xxh3_long_paths_identical_on_every_tier() {
    // Block edges for the default secret (1024 B blocks) and for a 200-byte
    // secret (17 stripes per block, so the generic partial-block loop runs).
    let lengths = [
        241usize, 255, 256, 257, 1023, 1024, 1025, 1087, 1088, 1089, 2048, 4096, 70_001,
    ];
    let secret = test_bytes(200, 5);
    let cases: Vec<Case> = lengths
        .iter()
        .map(|&len| {
            let data = test_bytes(len, len as u64);
            Case {
                h64: xxhash_rust::xxh3::xxh3_64(&data),
                h128: xxhash_rust::xxh3::xxh3_128(&data),
                seeded64: xxhash_rust::xxh3::xxh3_64_with_seed(&data, 99),
                seeded128: xxhash_rust::xxh3::xxh3_128_with_seed(&data, 99),
                secret64: xxhash_rust::xxh3::xxh3_64_with_secret(&data, &secret),
                secret128: xxhash_rust::xxh3::xxh3_128_with_secret(&data, &secret),
                data,
            }
        })
        .collect();
    let report = for_each_token_permutation(CompileTimePolicy::Warn, |perm| {
        for c in &cases {
            let d = &c.data;
            let len = d.len();
            assert_eq!(zenhash::xxh3_64(d), c.h64, "{perm}: len {len}");
            assert_eq!(zenhash::xxh3_128(d), c.h128, "{perm}: len {len}");
            assert_eq!(
                zenhash::xxh3_64_with_seed(d, 99),
                c.seeded64,
                "{perm}: len {len}"
            );
            assert_eq!(
                zenhash::xxh3_128_with_seed(d, 99),
                c.seeded128,
                "{perm}: len {len}"
            );
            assert_eq!(
                zenhash::xxh3_64_with_secret(d, &secret).unwrap(),
                c.secret64,
                "{perm}: len {len}"
            );
            assert_eq!(
                zenhash::xxh3_128_with_secret(d, &secret).unwrap(),
                c.secret128,
                "{perm}: len {len}"
            );
            // Streaming: chunk sizes that leave the block position mid-block
            // (partial-block paths) and that cross the 256-byte buffer.
            for chunk in [1usize, 63, 300, 4096] {
                let mut a = zenhash::Xxh3::new();
                let mut b = zenhash::Xxh3::with_seed(99);
                let mut s = zenhash::Xxh3::with_secret(&secret).unwrap();
                for part in d.chunks(chunk) {
                    a.update(part);
                    b.update(part);
                    s.update(part);
                }
                let ctx = format!("{perm}: streaming len {len} chunk {chunk}");
                assert_eq!(a.digest(), c.h64, "{ctx}");
                assert_eq!(a.digest128(), c.h128, "{ctx}");
                assert_eq!(b.digest(), c.seeded64, "{ctx}");
                assert_eq!(s.digest128(), c.secret128, "{ctx}");
            }
        }
    });
    eprintln!("{report}");
    // With `_dev` (all-features), every tier down to scalar is reachable.
    #[cfg(all(feature = "_dev", target_arch = "x86_64", feature = "avx512"))]
    assert!(report.permutations_run >= 4, "{report}");
    assert!(report.permutations_run >= 1, "{report}");
}
