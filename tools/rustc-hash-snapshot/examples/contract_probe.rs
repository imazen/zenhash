//! Report upstream output and write-boundary behavior without changing its algorithm.
use core::hash::Hasher;
use rustc_hash::FxHasher;

fn hash(parts: &[&[u8]]) -> u64 {
    let mut h = FxHasher::default();
    for part in parts {
        h.write(part);
    }
    h.finish()
}

fn main() {
    println!("pointer_width={}", usize::BITS);
    println!(
        "wide_arithmetic={}",
        cfg!(target_feature = "wide-arithmetic")
    );
    for bytes in [
        b"hello world".as_slice(),
        b"0123456789abcdef0123456789abcdef",
    ] {
        println!("input={}", core::str::from_utf8(bytes).unwrap());
        println!("whole={:016x}", hash(&[bytes]));
        for split in [1, 5, bytes.len() - 1] {
            println!(
                "split_{split}={:016x}",
                hash(&[&bytes[..split], &bytes[split..]])
            );
        }
        println!("empty_prefix={:016x}", hash(&[b"", bytes]));
        println!("empty_suffix={:016x}", hash(&[bytes, b""]));
    }
}
