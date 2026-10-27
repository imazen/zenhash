//! Full-width offset bases, including zero, maximum, high-bit and repeated-byte.
#[test]
fn full_width_bases_match_c() {
    use core::hash::Hasher;
    let mut rows = 0;
    for line in include_str!("../reference/fnv-bases.txt")
        .lines()
        .filter(|s| !s.starts_with('#'))
    {
        let mut words = line.split_whitespace();
        let pattern = words.next().unwrap().parse::<usize>().unwrap();
        let index = words.next().unwrap().parse::<usize>().unwrap();
        let a = [0u32, u32::MAX, 1 << 31, 0xa5a5a5a5][index];
        let b = [0u64, u64::MAX, 1 << 63, 0xa5a5a5a5a5a5a5a5][index];
        let c = [
            0u128,
            u128::MAX,
            1 << 127,
            0xa5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5,
        ][index];
        let mut states = [0xcbf29ce484222325u64; 3];
        for len in super::super::stability::fnv_lengths() {
            let data =
                super::super::stability::bytes(len, pattern, 0x9e3779b185ebca87 ^ len as u64);
            let split = (((len as u64).wrapping_mul(7) ^ index as u64) % (len as u64 + 1)) as usize;
            macro_rules! check {
                ($ty:ty,$basis:expr,$state:expr,$wide:expr) => {{
                    let mut h = <$ty>::with_basis($basis);
                    h.update(&data[..split]);
                    let prefix = h.digest();
                    let mut clone = h.clone();
                    h.update(&[]);
                    h.update(&data[split..]);
                    clone.write(&data[split..]);
                    let whole = h.digest();
                    assert_eq!(clone.digest(), whole);
                    let mut continuation = <$ty>::with_basis(prefix);
                    continuation.update(&data[split..]);
                    assert_eq!(continuation.digest(), whole);
                    h.reset();
                    assert_eq!(h.digest(), $basis);
                    #[cfg(feature = "io")]
                    {
                        std::io::Write::write_all(&mut h, &data).unwrap();
                        std::io::Write::flush(&mut h).unwrap();
                    }
                    #[cfg(not(feature = "io"))]
                    h.write(&data);
                    assert_eq!(h.digest(), whole);
                    for value in [whole as u128, prefix as u128] {
                        for word in [value as u64, (value >> 64) as u64]
                            .into_iter()
                            .take(if $wide { 2 } else { 1 })
                        {
                            for byte in word.to_le_bytes() {
                                $state = ($state ^ u64::from(byte)).wrapping_mul(0x100000001b3);
                            }
                        }
                    }
                }};
            }
            check!(zenhash::Fnv1a32, a, states[0], false);
            check!(zenhash::Fnv1a64, b, states[1], false);
            check!(zenhash::Fnv1a128, c, states[2], true);
        }
        for (actual, want) in states.into_iter().zip(words) {
            assert_eq!(actual, u64::from_str_radix(want, 16).unwrap());
        }
        rows += 1;
    }
    assert_eq!(rows, 48);
}
