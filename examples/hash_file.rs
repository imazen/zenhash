//! Prints the XXH3-64, XXH3-128, XXH64 and XXH32 hashes of a file, read in a
//! streaming fashion.
//!
//! `cargo run --release --example hash_file -- <path>`
use std::io::{BufReader, Read};

fn main() -> std::io::Result<()> {
    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("usage: hash_file <path>");
        std::process::exit(2);
    };
    let mut reader = BufReader::with_capacity(1 << 16, std::fs::File::open(&path)?);
    let mut xxh3 = zenhash::Xxh3::new();
    #[cfg(feature = "xxh64")]
    let mut xxh64 = zenhash::Xxh64::new();
    #[cfg(feature = "xxh32")]
    let mut xxh32 = zenhash::Xxh32::new();
    #[cfg(feature = "rapidhash")]
    let mut rapid = zenhash::RapidhashV3::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        xxh3.update(&buf[..n]);
        #[cfg(feature = "rapidhash")]
        rapid.update(&buf[..n]);
        #[cfg(feature = "xxh64")]
        xxh64.update(&buf[..n]);
        #[cfg(feature = "xxh32")]
        xxh32.update(&buf[..n]);
    }
    println!("XXH3-64   {:016x}", xxh3.digest());
    println!("XXH3-128  {:032x}", xxh3.digest128());
    #[cfg(feature = "xxh64")]
    println!("XXH64     {:016x}", xxh64.digest());
    #[cfg(feature = "xxh32")]
    println!("XXH32     {:08x}", xxh32.digest());
    #[cfg(feature = "rapidhash")]
    println!("rapidhash-v3 {:016x}", rapid.digest());
    Ok(())
}
