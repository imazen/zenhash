//! Compact C-reference fingerprints fix outputs across ISAs and SIMD tiers.
//! Every row covers all short lengths and long block/stripe/buffer boundaries.
use zenhash::{RapidhashV3, Xxh3, Xxh32, Xxh64};

const EDGES: &[usize] = &[
    271, 272, 273, 287, 288, 289, 319, 320, 321, 335, 336, 337, 511, 512, 513, 1000, 1023, 1024,
    1025, 1087, 1088, 1089, 2047, 2048, 2049, 4095, 4096, 4097, 10000, 16384, 65536,
];

fn bytes(len: usize, pattern: usize, mut seed: u64) -> Vec<u8> {
    (0..len)
        .map(|i| match pattern {
            0 => 0,
            1 => 255,
            2 => i as u8,
            3 => {
                if i % 2 == 0 {
                    0x55
                } else {
                    0xaa
                }
            }
            4 => {
                if i % 127 == 0 {
                    1 << (i % 8)
                } else {
                    0
                }
            }
            5 => {
                seed = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
                let mut z = seed;
                z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                (z ^ (z >> 31)) as u8
            }
            _ => panic!("unknown pattern {pattern}"),
        })
        .collect()
}

fn record(state: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *state = (*state ^ u64::from(byte)).wrapping_mul(0x100000001b3);
    }
}

pub(crate) fn verify() {
    let mut rows = 0;
    for line in include_str!("../reference/conformance.txt")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let fields: Vec<_> = line.split_whitespace().collect();
        assert_eq!(fields.len(), 11);
        let pattern: usize = fields[0].parse().unwrap();
        let seed = u64::from_str_radix(fields[1], 16).unwrap();
        let secret_len: usize = fields[2].parse().unwrap();
        let secret = bytes(secret_len, pattern, seed ^ secret_len as u64);
        let keys: [u64; 8] = core::array::from_fn(|i| {
            u64::from_le_bytes(secret[i * 8..i * 8 + 8].try_into().unwrap())
        });
        let mut states = [0xcbf29ce484222325; 8];
        for len in (0..=256).chain(EDGES.iter().copied()) {
            let data = bytes(len, pattern, seed ^ len as u64);
            let h32 = zenhash::xxh32(&data, seed as u32);
            let h64 = zenhash::xxh64(&data, seed);
            let h3 = zenhash::xxh3_64_with_seed(&data, seed);
            let h128 = zenhash::xxh3_128_with_seed(&data, seed);
            let s3 = zenhash::xxh3_64_with_secret(&data, &secret).unwrap();
            let s128 = zenhash::xxh3_128_with_secret(&data, &secret).unwrap();
            let rapid = zenhash::rapidhash_v3_with_seed(&data, seed);
            let rapid_secret = zenhash::rapidhash_v3_with_seed_and_secret(&data, seed, &keys);
            if seed == 0 {
                assert_eq!(zenhash::xxh3_64(&data), h3);
                assert_eq!(zenhash::xxh3_128(&data), h128);
                assert_eq!(zenhash::rapidhash_v3(&data), rapid);
            }
            for (index, value) in [
                u64::from(h32),
                h64,
                h3,
                h128 as u64,
                s3,
                s128 as u64,
                rapid,
                rapid_secret,
            ]
            .into_iter()
            .enumerate()
            {
                record(&mut states[index], value);
                if index == 3 {
                    record(&mut states[index], (h128 >> 64) as u64);
                }
                if index == 5 {
                    record(&mut states[index], (s128 >> 64) as u64);
                }
            }
            let split = (len.wrapping_mul(7) + pattern) % (len + 1);
            let mut a = Xxh32::with_seed(seed as u32);
            let mut b = Xxh64::with_seed(seed);
            let mut c = Xxh3::with_seed(seed);
            let mut d = Xxh3::with_secret(&secret).unwrap();
            let mut e = RapidhashV3::with_seed(seed);
            let mut f = RapidhashV3::with_seed_and_secret(seed, keys);
            let mut default3 = (seed == 0).then(Xxh3::new);
            for chunk in [&data[..split], &[], &data[split..]] {
                a.update(chunk);
                b.update(chunk);
                c.update(chunk);
                d.update(chunk);
                e.update(chunk);
                f.update(chunk);
                if let Some(h) = &mut default3 {
                    h.update(chunk);
                }
            }
            let ctx = format!(
                "pattern {pattern} seed {seed:x} secret {secret_len} len {len} split {split}"
            );
            assert_eq!(a.digest(), h32, "{ctx}");
            assert_eq!(b.digest(), h64, "{ctx}");
            assert_eq!(c.digest(), h3, "{ctx}");
            assert_eq!(c.digest128(), h128, "{ctx}");
            assert_eq!(d.digest(), s3, "{ctx}");
            assert_eq!(d.digest128(), s128, "{ctx}");
            assert_eq!(e.digest(), rapid, "{ctx}");
            assert_eq!(f.digest(), rapid_secret, "{ctx}");
            if let Some(h) = default3 {
                assert_eq!(h.digest(), h3, "{ctx}: default state");
                assert_eq!(h.digest128(), h128, "{ctx}: default state");
            }
            f.reset();
            f.update(&data);
            assert_eq!(f.digest(), rapid_secret, "{ctx}: reset preserves secret");
        }
        let want: Vec<_> = fields[3..]
            .iter()
            .map(|hex| u64::from_str_radix(hex, 16).unwrap())
            .collect();
        assert_eq!(
            states.as_slice(),
            want,
            "C reference pattern {pattern} seed {seed:x} secret {secret_len}"
        );
        rows += 1;
    }
    assert_eq!(rows, 144);
}
