//! Identical native/WASM benchmark batches. No unsafe export attributes: the
//! build script exports this function's compiler-generated symbol at link time.
#![forbid(unsafe_code)]
use core::hint::black_box;

pub const NAMES: &[&str] = &[
    "xxh3_64",
    "xxh3_128",
    "xxh3_64_seed",
    "xxh3_128_seed",
    "xxh3_64_secret",
    "xxh3_128_secret",
    "rapid_v3",
    "rapid_v3_seed",
    "rapid_v3_secret",
    "xxh32",
    "xxh64",
    "fnv32",
    "fnv64",
    "fnv128",
    "fnv64_fold32",
    "fnv64_fmix",
    "xxhash_rust_64",
    "xxhash_rust_128",
    "rapidhash_ref",
];

/// Hashes a deterministic byte slice repeatedly, with allocation amortized
/// over the batch. `chunk == 0` selects one-shot; otherwise uses streaming.
#[inline(never)]
pub extern "C" fn hash_batch(
    algorithm: u32,
    len: u32,
    seed: u64,
    chunk: u32,
    repetitions: u32,
) -> u64 {
    let data: Vec<u8> = (0..len)
        .map(|i| i.wrapping_mul(37).wrapping_add(i >> 8) as u8)
        .collect();
    let secret: [u8; 200] = core::array::from_fn(|i| (i as u8).wrapping_mul(79).wrapping_add(17));
    let keys: [u64; 8] =
        core::array::from_fn(|i| u64::from_le_bytes(secret[8 * i..8 * i + 8].try_into().unwrap()));
    macro_rules! run {
        ($one:expr, $ctor:expr, $digest:ident) => {
            run!($one, $ctor, $digest, update)
        };
        ($one:expr, $ctor:expr, $digest:ident, $write:ident) => {{
            let mut total = 0u64;
            for _ in 0..repetitions {
                let input = black_box(data.as_slice());
                let hash = if chunk == 0 {
                    ($one)(input) as u128
                } else {
                    let mut h = $ctor;
                    for part in input.chunks(chunk as usize) {
                        h.$write(part);
                    }
                    h.$digest() as u128
                };
                total = total.wrapping_add(black_box((hash as u64) ^ ((hash >> 64) as u64)));
            }
            total
        }};
    }
    match algorithm {
        0 => run!(zenhash::xxh3_64, zenhash::Xxh3::new(), digest),
        1 => run!(zenhash::xxh3_128, zenhash::Xxh3::new(), digest128),
        2 => run!(
            |d| zenhash::xxh3_64_with_seed(d, seed),
            zenhash::Xxh3::with_seed(seed),
            digest
        ),
        3 => run!(
            |d| zenhash::xxh3_128_with_seed(d, seed),
            zenhash::Xxh3::with_seed(seed),
            digest128
        ),
        4 => run!(
            |d| zenhash::xxh3_64_with_secret(d, &secret).unwrap(),
            zenhash::Xxh3::with_secret(&secret).unwrap(),
            digest
        ),
        5 => run!(
            |d| zenhash::xxh3_128_with_secret(d, &secret).unwrap(),
            zenhash::Xxh3::with_secret(&secret).unwrap(),
            digest128
        ),
        6 => run!(zenhash::rapidhash_v3, zenhash::RapidhashV3::new(), digest),
        7 => run!(
            |d| zenhash::rapidhash_v3_with_seed(d, seed),
            zenhash::RapidhashV3::with_seed(seed),
            digest
        ),
        8 => run!(
            |d| zenhash::rapidhash_v3_with_seed_and_secret(d, seed, &keys),
            zenhash::RapidhashV3::with_seed_and_secret(seed, keys),
            digest
        ),
        9 => run!(
            |d| zenhash::xxh32(d, seed as u32),
            zenhash::Xxh32::with_seed(seed as u32),
            digest
        ),
        10 => run!(
            |d| zenhash::xxh64(d, seed),
            zenhash::Xxh64::with_seed(seed),
            digest
        ),
        11 => run!(zenhash::fnv1a32, zenhash::Fnv1a32::new(), digest),
        12 => run!(zenhash::fnv1a64, zenhash::Fnv1a64::new(), digest),
        13 => run!(zenhash::fnv1a128, zenhash::Fnv1a128::new(), digest),
        14 => run!(
            zenhash::fnv1a64_fold32,
            zenhash::Fnv1a64::new(),
            digest_fold32
        ),
        15 => run!(zenhash::fnv1a64_fmix, zenhash::Fnv1a64::new(), digest_fmix),
        16 => run!(
            xxhash_rust::xxh3::xxh3_64,
            xxhash_rust::xxh3::Xxh3::new(),
            digest
        ),
        17 => run!(
            xxhash_rust::xxh3::xxh3_128,
            xxhash_rust::xxh3::Xxh3::new(),
            digest128
        ),
        18 => run!(
            rapidhash::v3::rapidhash_v3,
            rapidhash::v3::RapidStreamHasherV3::new(&rapidhash::v3::DEFAULT_RAPID_SECRETS),
            finish,
            write
        ),
        _ => panic!("unknown algorithm"),
    }
}
