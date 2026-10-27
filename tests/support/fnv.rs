//! Compatibility checks against an independent C unsigned-arithmetic reference.
use super::stability::{SEEDS, bytes};
use zenhash::{Fnv1a32, Fnv1a64, Fnv1a128};

#[test]
fn fnv_stable_across_streaming_and_offset_bases() {
    use core::hash::Hasher;
    let mut rows = 0;
    for line in include_str!("../reference/fnv.txt")
        .lines()
        .filter(|s| !s.starts_with('#'))
    {
        let mut words = line.split_whitespace();
        let pattern = words.next().unwrap().parse::<usize>().unwrap();
        let basis = u64::from_str_radix(words.next().unwrap(), 16).unwrap();
        assert!(SEEDS.contains(&basis));
        let mut states = [0xcbf29ce484222325u64; 5];
        for len in super::stability::fnv_lengths() {
            let data = bytes(len, pattern, basis ^ len as u64);
            let split = (((len as u64).wrapping_mul(7) ^ basis) % (len as u64 + 1)) as usize;
            macro_rules! check {
                ($ty:ty, $one:path) => {{
                    let mut h = if basis == 0 {
                        <$ty>::new()
                    } else {
                        <$ty>::with_basis(basis as _)
                    };
                    h.update(&data[..split]);
                    let prefix = h.digest();
                    let mut clone = h.clone();
                    h.update(&[]);
                    h.update(&data[split..]);
                    clone.write(&data[split..]);
                    let whole = h.digest();
                    assert_eq!(clone.digest(), whole);
                    assert_eq!(h.digest(), whole);
                    assert_eq!(h.finish(), whole as u64);
                    let mut continued = <$ty>::with_basis(prefix);
                    continued.update(&data[split..]);
                    assert_eq!(continued.digest(), whole);
                    let mut expected = if basis == 0 {
                        <$ty>::default()
                    } else {
                        <$ty>::with_basis(basis as _)
                    };
                    expected.write(&data);
                    assert_eq!(whole, expected.digest());
                    if basis == 0 {
                        assert_eq!(whole, $one(&data));
                    }
                    h.reset();
                    assert_eq!(
                        h.digest(),
                        if basis == 0 {
                            <$ty>::new().digest()
                        } else {
                            basis as _
                        }
                    );
                    #[cfg(feature = "io")]
                    {
                        std::io::Write::write_all(&mut h, &data).unwrap();
                        std::io::Write::flush(&mut h).unwrap();
                    }
                    #[cfg(not(feature = "io"))]
                    h.write(&data);
                    assert_eq!(h.digest(), whole);
                    (whole, prefix)
                }};
            }
            if len == 16 {
                let value = u128::from_le_bytes(data.as_slice().try_into().unwrap());
                super::stability::integers(Fnv1a32::new(), value);
                super::stability::integers(Fnv1a64::new(), value);
                super::stability::integers(Fnv1a128::new(), value);
            }
            let a = check!(Fnv1a32, zenhash::fnv1a32);
            let b = check!(Fnv1a64, zenhash::fnv1a64);
            let c = check!(Fnv1a128, zenhash::fnv1a128);
            for (a, b, c) in [(a.0, b.0, c.0), (a.1, b.1, c.1)] {
                let h = Fnv1a64::with_basis(b);
                let fold = h.digest_fold32();
                let mix = h.digest_fmix();
                assert_eq!(h.digest(), b);
                if basis == 0 && b == zenhash::fnv1a64(&data) {
                    assert_eq!(fold, zenhash::fnv1a64_fold32(&data));
                    assert_eq!(mix, zenhash::fnv1a64_fmix(&data));
                }
                for (state, values) in states.iter_mut().zip([
                    vec![u64::from(a)],
                    vec![b],
                    vec![c as u64, (c >> 64) as u64],
                    vec![u64::from(fold)],
                    vec![mix],
                ]) {
                    for value in values {
                        for byte in value.to_le_bytes() {
                            *state = (*state ^ u64::from(byte)).wrapping_mul(0x100000001b3);
                        }
                    }
                }
            }
        }
        for (actual, want) in states.into_iter().zip(words) {
            assert_eq!(
                actual,
                u64::from_str_radix(want, 16).unwrap(),
                "pattern {pattern} basis {basis:x}"
            );
        }
        rows += 1;
    }
    assert_eq!(rows, 48);
    assert_eq!(zenhash::fnv1a32(b"foobar"), 0xbf9cf968);
    assert_eq!(zenhash::fnv1a64(b"foobar"), 0x85944171f73967e8);
}

#[path = "fnv_bases.rs"]
mod full_bases;
