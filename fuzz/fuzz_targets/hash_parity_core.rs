/// Shared fuzz body, `include!`d by fuzz_targets/hash_parity.rs and by
/// tests/it/fuzz_regression.rs.
///
/// Layout: byte 0 picks the streaming chunk size, bytes 1..9 are the seed,
/// byte 9 picks how many leading bytes of the rest double as a custom secret.
/// The rest is the message. Checks one-shot against xxhash-rust and streaming
/// against one-shot, for every algorithm.
fn fuzz_hash_parity(data: &[u8]) {
    let Some((&[chunk_sel, s0, s1, s2, s3, s4, s5, s6, s7, secret_sel], msg)) =
        data.split_first_chunk::<10>()
    else {
        return;
    };
    let seed = u64::from_le_bytes([s0, s1, s2, s3, s4, s5, s6, s7]);
    let chunk = usize::from(chunk_sel) * 7 + 1;

    let want64 = xxhash_rust::xxh3::xxh3_64_with_seed(msg, seed);
    let want128 = xxhash_rust::xxh3::xxh3_128_with_seed(msg, seed);
    assert_eq!(zenhash::xxh3_64_with_seed(msg, seed), want64);
    assert_eq!(zenhash::xxh3_128_with_seed(msg, seed), want128);
    assert_eq!(zenhash::xxh64(msg, seed), xxhash_rust::xxh64::xxh64(msg, seed));
    assert_eq!(
        zenhash::xxh32(msg, seed as u32),
        xxhash_rust::xxh32::xxh32(msg, seed as u32)
    );

    let mut h3 = zenhash::Xxh3::with_seed(seed);
    let mut h64 = zenhash::Xxh64::with_seed(seed);
    let mut h32 = zenhash::Xxh32::with_seed(seed as u32);
    for c in msg.chunks(chunk) {
        h3.update(c);
        h64.update(c);
        h32.update(c);
    }
    assert_eq!(h3.digest(), want64);
    assert_eq!(h3.digest128(), want128);
    assert_eq!(h64.digest(), zenhash::xxh64(msg, seed));
    assert_eq!(h32.digest(), zenhash::xxh32(msg, seed as u32));

    // Custom secret: any length, including too-short ones that must be rejected.
    let secret = &msg[..msg.len().min(usize::from(secret_sel) + 100)];
    match zenhash::xxh3_64_with_secret(msg, secret) {
        Ok(h) => {
            assert!(secret.len() >= zenhash::XXH3_SECRET_SIZE_MIN);
            assert_eq!(h, xxhash_rust::xxh3::xxh3_64_with_secret(msg, secret));
            let mut s = zenhash::Xxh3::with_secret(secret).expect("validated above");
            for c in msg.chunks(chunk) {
                s.update(c);
            }
            assert_eq!(s.digest(), h);
            assert_eq!(
                s.digest128(),
                xxhash_rust::xxh3::xxh3_128_with_secret(msg, secret)
            );
        }
        Err(_) => assert!(secret.len() < zenhash::XXH3_SECRET_SIZE_MIN),
    }
}
