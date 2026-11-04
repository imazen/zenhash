//! Same-target compatibility with the exactly pinned upstream backend.
use core::hash::{BuildHasherDefault, Hash, Hasher};
use zenhash::nonportable::FxHasher;

type Reference = rustc_hash::FxHasher;

fn seeds() -> [usize; 7] {
    [
        0,
        1,
        usize::MAX,
        usize::MAX - 1,
        1 << (usize::BITS - 1),
        0x5555_5555,
        0xaaaa_aaaa,
    ]
}

#[test]
fn byte_writes_prefixes_and_clones_match_upstream() {
    let lengths: Vec<_> = (0..=257).chain([511, 512, 513, 1023, 1024, 4096]).collect();
    for seed in seeds() {
        for pattern in 0..8 {
            for &len in &lengths {
                let bytes: Vec<_> = (0..len)
                    .map(|i| match pattern {
                        0 => 0,
                        1 => 255,
                        2 => 0xa5,
                        3 => i as u8,
                        4 => ((i as u64).wrapping_mul(37) ^ ((i as u64) >> 8)) as u8,
                        5 => (seed.rotate_left((i % usize::BITS as usize) as u32)) as u8,
                        6 => 1 << (i % 8),
                        _ => b"shared-prefix-key/"[i % 18],
                    })
                    .collect();
                let mut actual = FxHasher::with_seed(seed);
                let mut reference = Reference::with_seed(seed);
                actual.write(&bytes);
                reference.write(&bytes);
                assert_eq!(actual.finish(), reference.finish(), "seed {seed} len {len}");
                let digest = actual.finish();
                assert_eq!(actual.finish(), digest);

                for split in [0, 1.min(len), len / 2, len] {
                    let mut actual = FxHasher::with_seed(seed);
                    let mut reference = Reference::with_seed(seed);
                    actual.write(&bytes[..split]);
                    reference.write(&bytes[..split]);
                    assert_eq!(actual.finish(), reference.finish());
                    let mut cloned = actual.clone();
                    let mut ref_cloned = reference.clone();
                    // Empty writes intentionally preserve upstream's call semantics.
                    actual.write(&[]);
                    reference.write(&[]);
                    actual.write(&bytes[split..]);
                    reference.write(&bytes[split..]);
                    cloned.write(&bytes[split..]);
                    ref_cloned.write(&bytes[split..]);
                    assert_eq!(actual.finish(), reference.finish());
                    assert_eq!(cloned.finish(), ref_cloned.finish());
                }
            }
        }
    }
}

#[test]
fn every_stable_integer_write_preserves_typed_semantics() {
    for seed in seeds() {
        for value in [
            0,
            1,
            u128::MAX,
            1 << 127,
            0x0123_4567_89ab_cdef_fedc_ba98_7654_3210,
        ] {
            let mut actual = FxHasher::with_seed(seed);
            let mut reference = Reference::with_seed(seed);
            macro_rules! check {
                ($method:ident, $ty:ty) => {
                    actual.$method(value as $ty);
                    reference.$method(value as $ty);
                    assert_eq!(actual.finish(), reference.finish(), stringify!($method));
                };
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
    }
}

#[test]
fn constructors_debug_and_real_hash_table_keys() {
    const NEW: FxHasher = FxHasher::new();
    const SEEDED: FxHasher = FxHasher::with_seed(7);
    assert_eq!(NEW.finish(), FxHasher::default().finish());
    assert_eq!(NEW.finish(), Reference::default().finish());
    assert_eq!(SEEDED.finish(), Reference::with_seed(7).finish());
    let mut debug = SEEDED.clone();
    debug.write(b"PRIVATE-INPUT");
    assert_eq!(format!("{debug:?}"), "FxHasher { .. }");

    #[derive(Hash, PartialEq, Eq)]
    struct Key {
        id: u64,
        category: i16,
        label: String,
    }
    let mut table = std::collections::HashMap::<Key, u64, BuildHasherDefault<FxHasher>>::default();
    for id in 0..64 {
        let key = Key {
            id,
            category: -(id as i16),
            label: format!("key/{id}"),
        };
        let mut actual = FxHasher::new();
        let mut reference = Reference::default();
        key.hash(&mut actual);
        key.hash(&mut reference);
        assert_eq!(actual.finish(), reference.finish());
        table.insert(key, id * 3);
    }
    for id in 0..64 {
        let key = Key {
            id,
            category: -(id as i16),
            label: format!("key/{id}"),
        };
        assert_eq!(table.remove(&key), Some(id * 3));
    }
    assert!(table.is_empty());
}
