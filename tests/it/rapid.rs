//! Standard v3 parity with the pinned C reference and hoxxep's Rust port.
use crate::test_bytes;
use rapidhash::v3::{RapidSecrets, RapidStreamHasherV3, rapidhash_v3_seeded};
use std::hash::Hasher;

const SEEDS: [u64; 4] = [0, 1, u64::MAX, 0x9e37_79b1_85eb_ca87];

fn oracle(data: &[u8], seed: u64) -> u64 {
    rapidhash_v3_seeded(data, &RapidSecrets::seed_cpp(seed))
}

#[test]
fn pinned_c_vectors() {
    let vectors = include_str!("../reference/rapidhash-v3.txt");
    let mut count = 0;
    for line in vectors.lines().filter(|line| !line.starts_with('#')) {
        let mut fields = line.split_whitespace();
        let len: usize = fields.next().unwrap().parse().unwrap();
        let seed = u64::from_str_radix(fields.next().unwrap(), 16).unwrap();
        let want = u64::from_str_radix(fields.next().unwrap(), 16).unwrap();
        assert!(fields.next().is_none());
        let data = test_bytes(len, len as u64);
        assert_eq!(
            zenhash::rapidhash_v3_with_seed(&data, seed),
            want,
            "C len {len} seed {seed}"
        );
        assert_eq!(
            oracle(&data, seed),
            want,
            "Rust oracle len {len} seed {seed}"
        );
        for chunk in [1, 7, 16, 111, 112, 113, 224, 4096] {
            let mut h = zenhash::RapidhashV3::with_seed(seed);
            for bytes in data.chunks(chunk) {
                h.update(b"");
                h.update(bytes);
            }
            assert_eq!(
                h.digest(),
                want,
                "C streaming len {len} seed {seed} chunk {chunk}"
            );
        }
        count += 1;
    }
    assert_eq!(count, 316);
}

#[test]
fn lengths_alignments_and_streaming_splits() {
    // Every remainder in a 112-byte block, multiple block counts, and unaligned reads.
    for len in 0..=1024 {
        let data = test_bytes(len + 7, len as u64);
        for offset in 0..8 {
            let input = &data[offset..offset + len];
            for seed in SEEDS {
                let want = oracle(input, seed);
                assert_eq!(
                    zenhash::rapidhash_v3_with_seed(input, seed),
                    want,
                    "len {len} offset {offset} seed {seed}"
                );
                if seed == 0 {
                    assert_eq!(zenhash::rapidhash_v3(input), want);
                }
            }
        }
        let data = &data[..len];
        for split in 0..=len.min(240) {
            let seed = len as u64;
            let want = oracle(data, seed);
            let secrets = RapidSecrets::seed_cpp(seed);
            let mut reference = RapidStreamHasherV3::new(&secrets);
            let mut h = zenhash::RapidhashV3::with_seed(seed);
            h.update(&data[..split]);
            reference.write(&data[..split]);
            assert_eq!(h.digest(), reference.finish(), "prefix {split}");
            // Repeated digest and empty writes must preserve state.
            assert_eq!(h.digest(), oracle(&data[..split], seed));
            h.update(b"");
            h.update(&data[split..]);
            reference.write(&data[split..]);
            assert_eq!(h.digest(), want, "len {len} split {split}");
            assert_eq!(h.digest(), reference.finish());
        }
    }
}

#[test]
fn random_chunks_clone_reset_and_traits() {
    for case in 0..500 {
        let len = (case * 7919) % 20003;
        let data = test_bytes(len, case as u64);
        let seed = (case as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        let want = oracle(&data, seed);
        let mut hasher = zenhash::RapidhashV3::with_seed(seed);
        let mut rest = &data[..];
        while !rest.is_empty() {
            let take = (1 + (rest.len() * 13 + case) % 337).min(rest.len());
            hasher.write(&rest[..take]);
            rest = &rest[take..];
        }
        assert_eq!(hasher.finish(), want, "case {case}");
        let mut clone = hasher.clone();
        clone.update(b"tail");
        assert_eq!(hasher.digest(), want);
        let mut extended = data.clone();
        extended.extend_from_slice(b"tail");
        assert_eq!(clone.digest(), oracle(&extended, seed));
        hasher.reset();
        assert_eq!(hasher.digest(), oracle(b"", seed));
        hasher.update(&data);
        assert_eq!(hasher.digest(), want);
    }
    let mut a = zenhash::RapidhashV3::new();
    let mut b = zenhash::RapidhashV3::default();
    a.write_u64(0x0102030405060708);
    b.update(&0x0102030405060708u64.to_ne_bytes());
    assert_eq!(a.finish(), b.finish());
    let mut h = zenhash::RapidhashV3::with_seed(u64::MAX);
    h.update(b"SENSITIVE");
    let debug = format!("{h:?}");
    assert!(debug.contains("total_len: 9"));
    assert!(!debug.contains("SENSITIVE") && !debug.contains(&u64::MAX.to_string()));
}

#[cfg(feature = "std")]
#[test]
fn io_write_matches_one_shot() {
    use std::io::Write;
    let data = test_bytes(10003, 4);
    let mut h = zenhash::RapidhashV3::with_seed(9);
    std::io::copy(&mut &data[..], &mut h).unwrap();
    assert_eq!(Write::write(&mut h, b"").unwrap(), 0);
    h.flush().unwrap();
    assert_eq!(h.digest(), oracle(&data, 9));
}
