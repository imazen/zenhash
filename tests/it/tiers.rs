//! Every SIMD tier the CPU can run must give the reference answer.

use archmage::testing::{CompileTimePolicy, for_each_token_permutation};

use crate::test_bytes;

#[test]
fn xxh3_long_paths_identical_on_every_tier() {
    let lengths = [
        241usize, 255, 256, 257, 1024, 1025, 1087, 1088, 1089, 4096, 70_001,
    ];
    let secret = test_bytes(200, 5);
    let inputs: Vec<(Vec<u8>, u64, u128, u64, u64)> = lengths
        .iter()
        .map(|&len| {
            let d = test_bytes(len, len as u64);
            let a = xxhash_rust::xxh3::xxh3_64(&d);
            let b = xxhash_rust::xxh3::xxh3_128(&d);
            let c = xxhash_rust::xxh3::xxh3_64_with_seed(&d, 99);
            let e = xxhash_rust::xxh3::xxh3_64_with_secret(&d, &secret);
            (d, a, b, c, e)
        })
        .collect();
    let report = for_each_token_permutation(CompileTimePolicy::Warn, |perm| {
        for (d, a, b, c, e) in &inputs {
            let len = d.len();
            assert_eq!(zenhash::xxh3_64(d), *a, "{perm}: len {len}");
            assert_eq!(zenhash::xxh3_128(d), *b, "{perm}: len {len}");
            assert_eq!(zenhash::xxh3_64_with_seed(d, 99), *c, "{perm}: len {len}");
            assert_eq!(
                zenhash::xxh3_64_with_secret(d, &secret).unwrap(),
                *e,
                "{perm}: len {len}"
            );
            let mut s = zenhash::Xxh3::new();
            for chunk in d.chunks(300) {
                s.update(chunk);
            }
            assert_eq!(s.digest(), *a, "{perm}: streaming len {len}");
        }
    });
    assert!(report.permutations_run >= 1, "{report}");
    eprintln!("{report}");
}
