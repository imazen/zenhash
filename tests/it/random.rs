//! Randomized differential tests: random lengths, seeds, secrets and update
//! splits, every algorithm, against xxhash-rust and twox-hash. Deterministic
//! (fixed PRNG seed), so a failure reproduces.

use twox_hash::{XxHash3_64, XxHash3_128};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        // splitmix64
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn bytes(&mut self, n: usize) -> Vec<u8> {
        (0..n).map(|_| self.next() as u8).collect()
    }
    /// Lengths weighted toward the path boundaries.
    fn len(&mut self) -> usize {
        match self.below(4) {
            0 => self.below(17) as usize,
            1 => self.below(241) as usize,
            2 => self.below(2048) as usize,
            _ => self.below(20_000) as usize,
        }
    }
}

#[test]
fn random_one_shot_and_streaming_match_oracles() {
    let mut rng = Rng(0x5EED);
    let mut total = 0usize;
    for case in 0..3000 {
        let len = rng.len();
        total += len;
        let data = rng.bytes(len);
        let seed = match rng.below(3) {
            0 => 0,
            1 => rng.below(256),
            _ => rng.next(),
        };
        let ctx = format!("case {case} len {len} seed {seed:#x}");

        let want64 = xxhash_rust::xxh3::xxh3_64_with_seed(&data, seed);
        let want128 = xxhash_rust::xxh3::xxh3_128_with_seed(&data, seed);
        assert_eq!(zenhash::xxh3_64_with_seed(&data, seed), want64, "{ctx}");
        assert_eq!(zenhash::xxh3_128_with_seed(&data, seed), want128, "{ctx}");
        assert_eq!(
            XxHash3_128::oneshot_with_seed(seed, &data),
            want128,
            "{ctx}"
        );
        assert_eq!(
            zenhash::xxh64(&data, seed),
            xxhash_rust::xxh64::xxh64(&data, seed),
            "{ctx}"
        );
        assert_eq!(
            zenhash::xxh32(&data, seed as u32),
            xxhash_rust::xxh32::xxh32(&data, seed as u32),
            "{ctx}"
        );

        // Streaming with random split points, including empty updates.
        let mut h3 = zenhash::Xxh3::with_seed(seed);
        let mut h64 = zenhash::Xxh64::with_seed(seed);
        let mut h32 = zenhash::Xxh32::with_seed(seed as u32);
        let mut rest = &data[..];
        while !rest.is_empty() {
            let max = [1u64, 64, 300, 5000][rng.below(4) as usize];
            let n = (rng.below(max + 1) as usize).min(rest.len());
            let (a, b) = rest.split_at(n);
            h3.update(a);
            h64.update(a);
            h32.update(a);
            rest = b;
            if rng.below(8) == 0 {
                // Digests mid-stream must not disturb the state.
                let _ = (h3.digest(), h3.digest128(), h64.digest(), h32.digest());
            }
        }
        assert_eq!(h3.digest(), want64, "{ctx} streaming");
        assert_eq!(h3.digest128(), want128, "{ctx} streaming");
        assert_eq!(
            h64.digest(),
            xxhash_rust::xxh64::xxh64(&data, seed),
            "{ctx} streaming"
        );
        assert_eq!(
            h32.digest(),
            xxhash_rust::xxh32::xxh32(&data, seed as u32),
            "{ctx} streaming"
        );
    }
    eprintln!("random_one_shot_and_streaming: {total} bytes");
    assert!(total > 3_000_000, "only {total} bytes hashed");
}

#[test]
fn random_custom_secrets_match_oracles() {
    let mut rng = Rng(0xC0FFEE);
    for case in 0..1000 {
        let secret_len = 136 + rng.below(400) as usize;
        let secret = rng.bytes(secret_len);
        let len = rng.len();
        let data = rng.bytes(len);
        let ctx = format!("case {case} len {len} secret {secret_len}");
        let want64 = xxhash_rust::xxh3::xxh3_64_with_secret(&data, &secret);
        let want128 = xxhash_rust::xxh3::xxh3_128_with_secret(&data, &secret);
        assert_eq!(
            zenhash::xxh3_64_with_secret(&data, &secret).unwrap(),
            want64,
            "{ctx}"
        );
        assert_eq!(
            zenhash::xxh3_128_with_secret(&data, &secret).unwrap(),
            want128,
            "{ctx}"
        );
        assert_eq!(
            XxHash3_64::oneshot_with_secret(&secret, &data).unwrap(),
            want64,
            "{ctx}"
        );

        let mut h = zenhash::Xxh3::with_secret(&secret).unwrap();
        let mut rest = &data[..];
        while !rest.is_empty() {
            let n = (rng.below(700) as usize).min(rest.len());
            let (a, b) = rest.split_at(n);
            h.update(a);
            rest = b;
        }
        assert_eq!(h.digest(), want64, "{ctx} streaming");
        assert_eq!(h.digest128(), want128, "{ctx} streaming");
    }
}
