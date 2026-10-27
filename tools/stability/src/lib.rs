//! Test executable kept outside the library's dev-dependency feature graph.
#![forbid(unsafe_code)]
#[cfg(test)]
#[cfg(any(
    feature = "xxh3",
    feature = "xxh32",
    feature = "xxh64",
    feature = "rapidhash",
    feature = "fnv"
))]
#[path = "../../../tests/support/stability.rs"]
mod stability;

#[test]
fn every_enabled_hash_matches_c() {
    #[cfg(any(
        feature = "xxh3",
        feature = "xxh32",
        feature = "xxh64",
        feature = "rapidhash"
    ))]
    stability::verify();
}

#[cfg(all(test, feature = "fnv"))]
#[path = "../../../tests/support/fnv.rs"]
mod fnv;
