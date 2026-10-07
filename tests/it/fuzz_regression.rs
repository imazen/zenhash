// The fuzz target and this replay share one body, so the two can't drift apart.
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/fuzz/fuzz_targets/hash_parity_core.rs"
));

/// Replays every committed seed in fuzz/regression/ on stable, on every CI platform.
/// Fails if the directory is missing or empty: a replay of zero seeds proves nothing.
#[test]
fn fuzz_regression_seeds_pass() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/regression");
    let mut replayed = 0;
    for entry in std::fs::read_dir(&dir).expect("fuzz/regression/ must exist") {
        let path = entry.expect("readable dir entry").path();
        if path.is_file() {
            fuzz_hash_parity(&std::fs::read(&path).expect("readable seed"));
            replayed += 1;
        }
    }
    assert!(
        replayed >= 2,
        "only {replayed} seeds replayed from {}",
        dir.display()
    );
}
