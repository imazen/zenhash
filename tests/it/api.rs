//! Public API behavior beyond hash values: traits, Debug hygiene, errors.

use std::hash::Hasher;

use crate::test_bytes;

#[test]
fn debug_never_shows_secret_seed_or_input() {
    let secret = [0xABu8; 200];
    let mut h = zenhash::Xxh3::with_secret(&secret).unwrap();
    h.update(b"SENSITIVE");
    let s = format!("{h:?}");
    assert!(s.contains("total_len: 9") && s.contains("custom"), "{s}");
    assert!(
        !s.contains("171") && !s.contains("SENSITIVE") && !s.contains("83"),
        "{s}"
    );

    let seed = 0x1234_5678_9ABC_DEF0u64;
    let mut h = zenhash::Xxh3::with_seed(seed);
    h.update(b"x");
    let s = format!("{h:?}");
    assert!(
        s.contains("seeded") && !s.contains(&seed.to_string()),
        "{s}"
    );
    assert!(format!("{:?}", zenhash::Xxh3::new()).contains("default"));

    let mut h = zenhash::Xxh64::with_seed(seed);
    h.update(b"SENSITIVE");
    let s = format!("{h:?}");
    assert!(
        s.contains("total_len: 9") && !s.contains(&seed.to_string()),
        "{s}"
    );
    let mut h = zenhash::Xxh32::with_seed(0xDEAD_BEEF);
    h.update(b"SENSITIVE");
    let s = format!("{h:?}");
    assert!(
        s.contains("total_len: 9") && !s.contains(&0xDEAD_BEEFu32.to_string()),
        "{s}"
    );
}

#[test]
fn new_default_and_seed_zero_agree() {
    let data = test_bytes(1000, 4);
    let mut hashers: [(Box<dyn Hasher>, u64); 9] = [
        (Box::new(zenhash::Xxh3::new()), zenhash::xxh3_64(&data)),
        (Box::new(zenhash::Xxh3::default()), zenhash::xxh3_64(&data)),
        (
            Box::new(zenhash::Xxh3::with_seed(0)),
            zenhash::xxh3_64(&data),
        ),
        (Box::new(zenhash::Xxh64::new()), zenhash::xxh64(&data, 0)),
        (
            Box::new(zenhash::Xxh64::default()),
            zenhash::xxh64(&data, 0),
        ),
        (
            Box::new(zenhash::Xxh64::with_seed(0)),
            zenhash::xxh64(&data, 0),
        ),
        (
            Box::new(zenhash::Xxh32::new()),
            u64::from(zenhash::xxh32(&data, 0)),
        ),
        (
            Box::new(zenhash::Xxh32::default()),
            u64::from(zenhash::xxh32(&data, 0)),
        ),
        (
            Box::new(zenhash::Xxh32::with_seed(0)),
            u64::from(zenhash::xxh32(&data, 0)),
        ),
    ];
    for (i, (h, want)) in hashers.iter_mut().enumerate() {
        h.write(&data);
        assert_eq!(h.finish(), *want, "hasher {i}");
    }
}

#[cfg(feature = "std")]
#[test]
fn io_write_matches_one_shot() {
    use std::io::Write;

    let data = test_bytes(10_000, 5);
    let mut a = zenhash::Xxh3::with_seed(9);
    let mut b = zenhash::Xxh64::with_seed(9);
    let mut c = zenhash::Xxh32::with_seed(9);
    std::io::copy(&mut &data[..], &mut a).unwrap();
    std::io::copy(&mut &data[..], &mut b).unwrap();
    std::io::copy(&mut &data[..], &mut c).unwrap();
    for w in [&mut a as &mut dyn Write, &mut b, &mut c] {
        assert_eq!(w.write(b"").unwrap(), 0);
        w.flush().unwrap();
    }
    assert_eq!(a.digest(), zenhash::xxh3_64_with_seed(&data, 9));
    assert_eq!(a.digest128(), zenhash::xxh3_128_with_seed(&data, 9));
    assert_eq!(b.digest(), zenhash::xxh64(&data, 9));
    assert_eq!(c.digest(), zenhash::xxh32(&data, 9));
}

#[test]
fn clones_continue_independently() {
    let data = test_bytes(3000, 6);
    let mut a = zenhash::Xxh3::with_secret(&test_bytes(150, 1)).unwrap();
    a.update(&data[..1700]);
    let mut b = a.clone();
    a.update(&data[1700..]);
    b.update(b"different tail");
    let mut whole = zenhash::Xxh3::with_secret(&test_bytes(150, 1)).unwrap();
    whole.update(&data);
    assert_eq!(a.digest(), whole.digest());
    assert_ne!(a.digest(), b.digest());

    let mut x = zenhash::Xxh64::new();
    x.update(&data[..33]);
    let y = x.clone();
    assert_eq!(x.digest(), y.digest());
    let mut x = zenhash::Xxh32::new();
    x.update(&data[..17]);
    let y = x.clone();
    assert_eq!(x.digest(), y.digest());
}

#[test]
fn reset_keeps_seed_and_secret() {
    let data = test_bytes(2000, 7);
    let secret = test_bytes(140, 2);
    let mut h = zenhash::Xxh3::with_secret(&secret).unwrap();
    h.update(&data);
    h.reset();
    h.update(&data[..500]);
    assert_eq!(
        h.digest(),
        zenhash::xxh3_64_with_secret(&data[..500], &secret).unwrap()
    );
    assert_eq!(
        h.digest128(),
        zenhash::xxh3_128_with_secret(&data[..500], &secret).unwrap()
    );

    let mut h = zenhash::Xxh32::with_seed(77);
    h.update(&data);
    h.reset();
    h.update(&data[..20]);
    assert_eq!(h.digest(), zenhash::xxh32(&data[..20], 77));
}

#[test]
fn error_is_descriptive_and_typed() {
    for len in [0usize, 1, 135] {
        let secret = vec![1u8; len];
        let err = zenhash::xxh3_128_with_secret(b"abc", &secret).unwrap_err();
        assert_eq!(*err.error(), zenhash::Error::SecretTooShort { len });
        let msg = err.error().to_string();
        assert!(
            msg.contains(&len.to_string()) && msg.contains("136"),
            "{msg}"
        );
        let dyn_err: &dyn std::error::Error = err.error();
        assert!(dyn_err.source().is_none());
    }
    assert!(zenhash::xxh3_64_with_secret(b"", &[0u8; 136]).is_ok());
}

#[test]
fn hasher_integer_writes_are_native_endian_bytes() {
    // Documented behavior: integer writes feed native-endian bytes.
    let mut a = zenhash::Xxh3::new();
    a.write_u64(0x0102_0304_0506_0708);
    assert_eq!(
        a.finish(),
        zenhash::xxh3_64(&0x0102_0304_0506_0708u64.to_ne_bytes())
    );
}

/// Documented on `xxh3_128_with_seed`: the 128-bit low half equals the 64-bit
/// hash only for inputs of 1..=3 bytes or over 240 bytes.
#[test]
fn xxh3_128_low_half_matches_64_only_where_documented() {
    for len in (0..=240usize).filter(|l| !(1..=3).contains(l)) {
        let d = test_bytes(len, 8);
        assert_ne!(
            zenhash::xxh3_128(&d) as u64,
            zenhash::xxh3_64(&d),
            "len {len}"
        );
    }
    for len in [1usize, 2, 3, 241, 1000, 5000] {
        let d = test_bytes(len, 8);
        assert_eq!(
            zenhash::xxh3_128(&d) as u64,
            zenhash::xxh3_64(&d),
            "len {len}"
        );
        assert_eq!(
            zenhash::xxh3_128_with_seed(&d, 5) as u64,
            zenhash::xxh3_64_with_seed(&d, 5),
            "len {len}"
        );
    }
}
