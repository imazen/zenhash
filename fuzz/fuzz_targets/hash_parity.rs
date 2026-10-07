#![no_main]
use libfuzzer_sys::fuzz_target;

include!("hash_parity_core.rs");

fuzz_target!(|data: &[u8]| fuzz_hash_parity(data));
