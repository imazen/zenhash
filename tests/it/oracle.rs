//! Parity with two independent implementations: xxhash-rust and twox-hash.

use crate::{interesting_lengths, test_bytes};
use twox_hash::{XxHash3_64, XxHash3_128, XxHash32, XxHash64};

const SEEDS: [u64; 4] = [0, 1, 0x9E37_79B1_85EB_CA87, u64::MAX];

#[test]
fn xxh3_64_matches_oracles() {
    for len in interesting_lengths() {
        let data = test_bytes(len, len as u64);
        assert_eq!(
            zenhash::xxh3_64(&data),
            xxhash_rust::xxh3::xxh3_64(&data),
            "len {len}"
        );
        for seed in SEEDS {
            let ours = zenhash::xxh3_64_with_seed(&data, seed);
            assert_eq!(
                ours,
                xxhash_rust::xxh3::xxh3_64_with_seed(&data, seed),
                "len {len} seed {seed}"
            );
            assert_eq!(
                ours,
                XxHash3_64::oneshot_with_seed(seed, &data),
                "len {len} seed {seed}"
            );
        }
    }
}

#[test]
fn xxh3_128_matches_oracles() {
    for len in interesting_lengths() {
        let data = test_bytes(len, !(len as u64));
        assert_eq!(
            zenhash::xxh3_128(&data),
            xxhash_rust::xxh3::xxh3_128(&data),
            "len {len}"
        );
        for seed in SEEDS {
            let ours = zenhash::xxh3_128_with_seed(&data, seed);
            assert_eq!(
                ours,
                xxhash_rust::xxh3::xxh3_128_with_seed(&data, seed),
                "len {len} seed {seed}"
            );
            assert_eq!(
                ours,
                XxHash3_128::oneshot_with_seed(seed, &data),
                "len {len} seed {seed}"
            );
        }
    }
}

#[test]
fn xxh3_custom_secret_matches_oracles() {
    // 136 is the minimum; 137 and 200 give block sizes that don't divide evenly;
    // 192 is the default size; 1000 gives long blocks.
    for secret_len in [136usize, 137, 192, 200, 1000] {
        let secret = test_bytes(secret_len, 77 + secret_len as u64);
        for len in [
            0usize, 1, 3, 4, 8, 9, 16, 17, 128, 129, 240, 241, 1000, 5000, 20_000,
        ] {
            let data = test_bytes(len, len as u64 + 5);
            let ours64 = zenhash::xxh3_64_with_secret(&data, &secret).unwrap();
            assert_eq!(
                ours64,
                xxhash_rust::xxh3::xxh3_64_with_secret(&data, &secret),
                "secret {secret_len} len {len}"
            );
            assert_eq!(
                ours64,
                XxHash3_64::oneshot_with_secret(&secret, &data).unwrap(),
                "secret {secret_len} len {len}"
            );
            let ours128 = zenhash::xxh3_128_with_secret(&data, &secret).unwrap();
            assert_eq!(
                ours128,
                xxhash_rust::xxh3::xxh3_128_with_secret(&data, &secret),
                "secret {secret_len} len {len}"
            );
        }
    }
}

#[test]
fn short_secret_is_rejected() {
    let secret = [7u8; 135];
    let err = zenhash::xxh3_64_with_secret(b"abc", &secret).unwrap_err();
    assert_eq!(*err.error(), zenhash::Error::SecretTooShort { len: 135 });
    assert!(zenhash::xxh3_128_with_secret(b"abc", &secret).is_err());
    assert!(zenhash::Xxh3::with_secret(&secret).is_err());
}

#[test]
fn xxh64_and_xxh32_match_oracles() {
    for len in interesting_lengths() {
        let data = test_bytes(len, len as u64 * 3);
        for seed in SEEDS {
            let ours = zenhash::xxh64(&data, seed);
            assert_eq!(ours, xxhash_rust::xxh64::xxh64(&data, seed), "len {len}");
            assert_eq!(ours, XxHash64::oneshot(seed, &data), "len {len}");
            let seed32 = seed as u32;
            let ours = zenhash::xxh32(&data, seed32);
            assert_eq!(ours, xxhash_rust::xxh32::xxh32(&data, seed32), "len {len}");
            assert_eq!(ours, XxHash32::oneshot(seed32, &data), "len {len}");
        }
    }
}

/// Feeds `data` in chunks of the given sizes, cycling.
fn chunked(data: &[u8], sizes: &[usize], mut f: impl FnMut(&[u8])) {
    let mut rest = data;
    let mut i = 0;
    while !rest.is_empty() {
        let n = sizes[i % sizes.len()].min(rest.len());
        let (a, b) = rest.split_at(n);
        f(a);
        rest = b;
        i += 1;
    }
}

const SPLITS: &[&[usize]] = &[
    &[1],
    &[7],
    &[63, 1],
    &[64],
    &[255],
    &[256],
    &[257],
    &[1, 300, 0, 64],
    &[1000],
    &[4096, 3],
];

#[test]
fn streaming_matches_one_shot() {
    let mut lengths = interesting_lengths();
    lengths.retain(|&l| l <= 20_000);
    let secret = test_bytes(171, 9);
    for len in lengths {
        let data = test_bytes(len, len as u64 + 11);
        let want64 = zenhash::xxh3_64(&data);
        let want128 = zenhash::xxh3_128(&data);
        let want_seeded = zenhash::xxh3_64_with_seed(&data, 42);
        let want_seeded128 = zenhash::xxh3_128_with_seed(&data, 42);
        let want_secret = zenhash::xxh3_64_with_secret(&data, &secret).unwrap();
        let want_secret128 = zenhash::xxh3_128_with_secret(&data, &secret).unwrap();
        let want_xxh64 = zenhash::xxh64(&data, 42);
        let want_xxh32 = zenhash::xxh32(&data, 42);
        for split in SPLITS {
            let mut a = zenhash::Xxh3::new();
            let mut b = zenhash::Xxh3::with_seed(42);
            let mut c = zenhash::Xxh3::with_secret(&secret).unwrap();
            let mut d = zenhash::Xxh64::with_seed(42);
            let mut e = zenhash::Xxh32::with_seed(42);
            chunked(&data, split, |chunk| {
                a.update(chunk);
                b.update(chunk);
                c.update(chunk);
                d.update(chunk);
                e.update(chunk);
            });
            let ctx = format!("len {len} split {split:?}");
            assert_eq!(a.digest(), want64, "{ctx}");
            assert_eq!(a.digest128(), want128, "{ctx}");
            assert_eq!(b.digest(), want_seeded, "{ctx}");
            assert_eq!(b.digest128(), want_seeded128, "{ctx}");
            assert_eq!(c.digest(), want_secret, "{ctx}");
            assert_eq!(c.digest128(), want_secret128, "{ctx}");
            assert_eq!(d.digest(), want_xxh64, "{ctx}");
            assert_eq!(e.digest(), want_xxh32, "{ctx}");
        }
    }
}

#[test]
fn digest_does_not_consume_and_reset_restarts() {
    let data = test_bytes(5000, 1);
    let mut h = zenhash::Xxh3::with_seed(3);
    h.update(&data[..2500]);
    let mid = h.digest();
    assert_eq!(mid, zenhash::xxh3_64_with_seed(&data[..2500], 3));
    h.update(&data[2500..]);
    assert_eq!(h.digest(), zenhash::xxh3_64_with_seed(&data, 3));
    h.reset();
    h.update(&data[..100]);
    assert_eq!(h.digest(), zenhash::xxh3_64_with_seed(&data[..100], 3));

    let mut h = zenhash::Xxh64::with_seed(3);
    h.update(&data);
    h.reset();
    h.update(&data[..40]);
    assert_eq!(h.digest(), zenhash::xxh64(&data[..40], 3));
}

#[test]
fn hasher_trait_matches_oracle_hasher() {
    use core::hash::Hasher;
    let data = test_bytes(3000, 2);
    let mut ours = zenhash::Xxh3::default();
    let mut theirs = xxhash_rust::xxh3::Xxh3::default();
    for chunk in data.chunks(97) {
        ours.write(chunk);
        theirs.write(chunk);
    }
    assert_eq!(ours.finish(), theirs.finish());
}
