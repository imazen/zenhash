//! Prints the XXH3-64, XXH3-128, XXH64 and XXH32 hashes of a file, read in a
//! streaming fashion.
//!
//! `cargo run --release --example hash_file -- <path>`
use std::io::{BufReader, Read, Write};

fn main() -> std::io::Result<()> {
    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("usage: hash_file <path>");
        std::process::exit(2);
    };
    let mut reader = BufReader::with_capacity(1 << 16, std::fs::File::open(&path)?);
    let mut xxh3 = zenhash::Xxh3::new();
    let mut xxh64 = zenhash::Xxh64::new();
    let mut xxh32 = zenhash::Xxh32::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        // All three hashers implement std::io::Write; update() is the same.
        xxh3.write_all(&buf[..n])?;
        xxh64.update(&buf[..n]);
        xxh32.update(&buf[..n]);
    }
    println!("XXH3-64   {:016x}", xxh3.digest());
    println!("XXH3-128  {:032x}", xxh3.digest128());
    println!("XXH64     {:016x}", xxh64.digest());
    println!("XXH32     {:08x}", xxh32.digest());
    Ok(())
}
