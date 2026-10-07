//! Published values, independent of any Rust implementation.

/// From the xxHash README / reference test suite: hashes of the empty input.
#[test]
fn empty_input() {
    assert_eq!(zenhash::xxh32(b"", 0), 0x02CC_5D05);
    assert_eq!(zenhash::xxh64(b"", 0), 0xEF46_DB37_51D8_E999);
    assert_eq!(zenhash::xxh3_64(b""), 0x2D06_8005_38D3_94C2);
    assert_eq!(
        zenhash::xxh3_128(b""),
        0x99AA_06D3_0147_98D8_6001_C324_468D_497F
    );
}
