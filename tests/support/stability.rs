//! Feature-independent stability runner shared by native, WASM and tier tests.

#[cfg(any(
    feature = "xxh3",
    feature = "xxh32",
    feature = "xxh64",
    feature = "rapidhash"
))]
#[inline]
fn record(state: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *state = (*state ^ u64::from(byte)).wrapping_mul(0x100000001b3);
    }
}

// Each state is the fingerprint of one C-reference output variant. Disabled
// algorithms are absent at compile time, including from the dependency graph.
#[cfg(any(
    feature = "xxh3",
    feature = "xxh32",
    feature = "xxh64",
    feature = "rapidhash"
))]
fn exercise(data: &[u8], seed: u64, secret: &[u8], states: &mut [u64; 8], prefix: bool) {
    let split = (((data.len() as u64).wrapping_mul(7) ^ seed ^ secret.len() as u64)
        % (data.len() as u64 + 1)) as usize;
    macro_rules! stream {
        ($ctor:expr, $one:expr, $digest:ident) => {{
            let mut h = $ctor;
            h.update(&data[..split]);
            let mid = ($one)(&data[..split]);
            assert_eq!(h.$digest(), mid, "prefix len {} seed {seed:x}", data.len());
            let mut clone = h.clone();
            h.update(&[]);
            h.update(&data[split..]);
            std::hash::Hasher::write(&mut clone, &data[split..]);
            let want = ($one)(data);
            assert_eq!(h.$digest(), want, "stream len {} seed {seed:x}", data.len());
            assert_eq!(
                clone.$digest(),
                want,
                "clone len {} seed {seed:x}",
                data.len()
            );
            assert_eq!(std::hash::Hasher::finish(&h), u64::from(h.digest()));
            assert_eq!(h.$digest(), want, "repeated digest");
            h.reset();
            assert_eq!(h.$digest(), ($one)(&[]), "reset seed/secret");
            #[cfg(feature = "io")]
            {
                std::io::Write::write_all(&mut h, data).unwrap();
                std::io::Write::flush(&mut h).unwrap();
            }
            #[cfg(not(feature = "io"))]
            std::hash::Hasher::write(&mut h, data);
            assert_eq!(
                h.$digest(),
                want,
                "reset then write len {} seed {seed:x}",
                data.len()
            );
            if data.len() == 16 {
                integers($ctor, u128::from_le_bytes(data.try_into().unwrap()));
            }
            (want, mid)
        }};
    }
    #[cfg(feature = "xxh32")]
    {
        let (want, mid) = stream!(
            zenhash::Xxh32::with_seed(seed as u32),
            |d: &[u8]| zenhash::xxh32(d, seed as u32),
            digest
        );
        record(&mut states[0], u64::from(want));
        if prefix {
            record(&mut states[0], u64::from(mid));
        }
    }
    #[cfg(feature = "xxh64")]
    {
        let (want, mid) = stream!(
            zenhash::Xxh64::with_seed(seed),
            |d: &[u8]| zenhash::xxh64(d, seed),
            digest
        );
        record(&mut states[1], want);
        if prefix {
            record(&mut states[1], mid);
        }
    }
    #[cfg(feature = "xxh3")]
    {
        let (want, mid) = stream!(
            zenhash::Xxh3::with_seed(seed),
            |d: &[u8]| zenhash::xxh3_64_with_seed(d, seed),
            digest
        );
        record(&mut states[2], want);
        if prefix {
            record(&mut states[2], mid);
        }
        let (want128, mid128) = stream!(
            zenhash::Xxh3::with_seed(seed),
            |d: &[u8]| zenhash::xxh3_128_with_seed(d, seed),
            digest128
        );
        for value in [want128, mid128]
            .into_iter()
            .take(if prefix { 2 } else { 1 })
        {
            record(&mut states[3], value as u64);
            record(&mut states[3], (value >> 64) as u64);
        }
        let (custom, custom_mid) = stream!(
            zenhash::Xxh3::with_secret(secret).unwrap(),
            |d: &[u8]| zenhash::xxh3_64_with_secret(d, secret).unwrap(),
            digest
        );
        record(&mut states[4], custom);
        if prefix {
            record(&mut states[4], custom_mid);
        }
        let (custom128, custom_mid128) = stream!(
            zenhash::Xxh3::with_secret(secret).unwrap(),
            |d: &[u8]| zenhash::xxh3_128_with_secret(d, secret).unwrap(),
            digest128
        );
        for value in [custom128, custom_mid128]
            .into_iter()
            .take(if prefix { 2 } else { 1 })
        {
            record(&mut states[5], value as u64);
            record(&mut states[5], (value >> 64) as u64);
        }
        if seed == 0 {
            assert_eq!(zenhash::xxh3_64(data), want);
            assert_eq!(zenhash::xxh3_128(data), want128);
            let mut h = zenhash::Xxh3::new();
            h.update(data);
            assert_eq!(h.digest(), want);
            assert_eq!(h.digest128(), want128);
        }
    }
    #[cfg(feature = "rapidhash")]
    {
        let keys: [u64; 8] = core::array::from_fn(|i| {
            u64::from_le_bytes(secret[i * 8..i * 8 + 8].try_into().unwrap())
        });
        let (want, mid) = stream!(
            zenhash::RapidhashV3::with_seed(seed),
            |d: &[u8]| zenhash::rapidhash_v3_with_seed(d, seed),
            digest
        );
        record(&mut states[6], want);
        if prefix {
            record(&mut states[6], mid);
        }
        let (custom, custom_mid) = stream!(
            zenhash::RapidhashV3::with_seed_and_secret(seed, keys),
            |d: &[u8]| zenhash::rapidhash_v3_with_seed_and_secret(d, seed, &keys),
            digest
        );
        record(&mut states[7], custom);
        if prefix {
            record(&mut states[7], custom_mid);
        }
        if seed == 0 {
            assert_eq!(zenhash::rapidhash_v3(data), want);
        }
    }
}

#[cfg(any(
    feature = "xxh3",
    feature = "xxh32",
    feature = "xxh64",
    feature = "rapidhash",
    feature = "fnv"
))]
pub(crate) fn integers<H: std::hash::Hasher + Clone>(base: H, value: u128) {
    macro_rules! check {
        ($method:ident, $ty:ty) => {{
            let mut h = base.clone();
            let mut bytes = base.clone();
            h.$method(value as $ty);
            bytes.write(&(value as $ty).to_ne_bytes());
            assert_eq!(h.finish(), bytes.finish(), stringify!($method));
        }};
    }
    check!(write_u8, u8);
    check!(write_u16, u16);
    check!(write_u32, u32);
    check!(write_u64, u64);
    check!(write_u128, u128);
    check!(write_usize, usize);
    check!(write_i8, i8);
    check!(write_i16, i16);
    check!(write_i32, i32);
    check!(write_i64, i64);
    check!(write_i128, i128);
    check!(write_isize, isize);
}

pub(crate) const SEEDS: [u64; 4] = [0, 1, u64::MAX, 0x9e3779b185ebca87];
const EDGES: &[usize] = &[
    271, 272, 273, 287, 288, 289, 319, 320, 321, 335, 336, 337, 511, 512, 513, 1000, 1023, 1024,
    1025, 1087, 1088, 1089, 2047, 2048, 2049, 4095, 4096, 4097, 10000, 16384, 65536,
];

pub(crate) fn bytes(len: usize, pattern: usize, mut seed: u64) -> Vec<u8> {
    let text = b"user/0000000000000000/item";
    let cycles = [3usize, 7, 17];
    let mut words = [0u32, 1, 0x80000000, u32::MAX];
    let mut rank = seed % 24;
    for i in 0..4 {
        let n = 4 - i;
        let j = i + rank as usize % n;
        words.swap(i, j);
        rank /= n as u64;
    }
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
                seed = seed.wrapping_add(0x9e3779b97f4a7c15);
                let mut z = seed;
                z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
                (z ^ (z >> 31)) as u8
            }
            6 => {
                let p = i % text.len();
                if (5..21).contains(&p) {
                    b"0123456789abcdef"[((seed >> ((20 - p) * 4)) & 15) as usize]
                } else {
                    text[p]
                }
            }
            7..=9 => {
                let p = i % cycles[pattern - 7];
                (p as u8)
                    .wrapping_mul(37)
                    .wrapping_add((seed >> (p % 8 * 8)) as u8)
            }
            10 => (words[(i / 4) % 4] >> ((i % 4) * 8)) as u8,
            11 => {
                if (i / 8) % 2 == 0 {
                    (seed >> (i % 8 * 8)) as u8
                } else {
                    !(seed >> (i % 8 * 8)) as u8
                }
            }
            _ => panic!("unknown pattern {pattern}"),
        })
        .collect()
}

#[cfg(any(
    feature = "xxh3",
    feature = "xxh32",
    feature = "xxh64",
    feature = "rapidhash"
))]
fn expected(line: &str, states: &[u64; 8]) {
    let values: Vec<_> = line.split_whitespace().collect();
    let active = [
        cfg!(feature = "xxh32"),
        cfg!(feature = "xxh64"),
        cfg!(feature = "xxh3"),
        cfg!(feature = "xxh3"),
        cfg!(feature = "xxh3"),
        cfg!(feature = "xxh3"),
        cfg!(feature = "rapidhash"),
        cfg!(feature = "rapidhash"),
    ];
    for i in 0..8 {
        if active[i] {
            assert_eq!(
                states[i],
                u64::from_str_radix(values[i], 16).unwrap(),
                "C fingerprint variant {i}"
            );
        }
    }
}

#[cfg(any(
    feature = "xxh3",
    feature = "xxh32",
    feature = "xxh64",
    feature = "rapidhash"
))]
pub(crate) fn verify() {
    // Retain every pre-existing C expectation as well as the expanded corpus.
    let mut old_rows = 0;
    for line in include_str!("../reference/conformance.txt")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let mut f = line.split_whitespace();
        let pattern: usize = f.next().unwrap().parse().unwrap();
        let seed = u64::from_str_radix(f.next().unwrap(), 16).unwrap();
        let size: usize = f.next().unwrap().parse().unwrap();
        let secret = bytes(size, pattern, seed ^ size as u64);
        let mut states = [0xcbf29ce484222325; 8];
        for len in (0..=256).chain(EDGES.iter().copied()) {
            exercise(
                &bytes(len, pattern, seed ^ len as u64),
                seed,
                &secret,
                &mut states,
                false,
            );
        }
        expected(&f.collect::<Vec<_>>().join(" "), &states);
        old_rows += 1;
    }
    assert_eq!(old_rows, 144);
    let mut rows = 0;
    for line in include_str!("../reference/conformance-extended.txt")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let mut f = line.split_whitespace();
        let family: usize = f.next().unwrap().parse().unwrap();
        let size: usize = f.next().unwrap().parse().unwrap();
        let mut states = [0xcbf29ce484222325; 8];
        extended(family, size, |data, seed, secret| {
            exercise(data, seed, secret, &mut states, true)
        });
        expected(&f.collect::<Vec<_>>().join(" "), &states);
        rows += 1;
    }
    assert_eq!(rows, 75);
}

#[cfg(any(
    feature = "xxh3",
    feature = "xxh32",
    feature = "xxh64",
    feature = "rapidhash"
))]
pub(crate) fn extended(mut family: usize, size: usize, mut f: impl FnMut(&[u8], u64, &[u8])) {
    let special = family >= 12;
    if special {
        family -= 12;
    }
    let mut lengths: Vec<usize> = (0..=256).chain(EDGES.iter().copied()).collect();
    let block = ((size - 64) / 8) * 64;
    for base in [block, block * 2] {
        lengths.extend([base - 1, base, base + 1]);
    }
    lengths.sort_unstable();
    lengths.dedup();
    for secret_pattern in 0..6 {
        for seed in SEEDS {
            let secret = bytes(size, secret_pattern, seed ^ size as u64);
            if !special {
                for &len in &lengths {
                    f(&bytes(len, family, seed ^ len as u64), seed, &secret);
                }
            } else if family == 0 {
                for len in [1, 4, 8, 16, 31, 64, 112, 240, 241, 1024] {
                    let mut data = bytes(len, 5, seed ^ len as u64);
                    for bit in 0..len * 8 {
                        data[bit / 8] ^= 1 << (bit % 8);
                        f(&data, seed, &secret);
                        data[bit / 8] ^= 1 << (bit % 8);
                    }
                }
            } else if family == 1 {
                for bit in 0..64 {
                    for seed in [
                        1u64 << bit,
                        !(1u64 << bit),
                        (1u64 << bit) | (1u64 << ((bit + 1) % 64)),
                    ] {
                        for len in [0, 8, 12, 15, 16, 17, 32, 64, 128, 241, 1024] {
                            for offset in [0usize, 1, 4, 8, 15] {
                                let mut data = vec![0; len];
                                for (i, byte) in seed.to_le_bytes().into_iter().enumerate() {
                                    if offset + i < len {
                                        data[offset + i] = byte;
                                    }
                                }
                                f(&data, seed, &secret);
                            }
                        }
                    }
                }
            } else {
                for rank in 0..24 {
                    for &len in &lengths {
                        f(&bytes(len, 10, rank), seed, &secret);
                    }
                }
            }
        }
    }
}

#[cfg(feature = "fnv")]
pub(crate) fn fnv_lengths() -> impl Iterator<Item = usize> {
    (0..=256).chain(EDGES.iter().copied())
}
