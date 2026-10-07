//! zenhash vs twox-hash vs xxhash-rust, one-shot and streaming.
//!
//! `cargo bench --bench hash`. Each input size is its own group (throughput is
//! per group); benches in a group run interleaved and are compared pairwise
//! against the group's first bench, zenhash.
//!
//! `ZENHASH_BENCH=<substring>` runs only the groups whose name contains it,
//! e.g. `ZENHASH_BENCH=xxh3_64/240 cargo bench --bench hash`.
use std::hash::Hasher;

use zenbench::prelude::*;

/// Bench closures must be `'static`; leaking a few MiB once per run is fine.
fn data(len: usize) -> &'static [u8] {
    let mut state = 0x1234_5678_9ABC_DEF0u64;
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state as u8
        })
        .collect::<Vec<u8>>()
        .leak()
}

const SIZES: [usize; 8] = [8, 16, 64, 128, 240, 1024, 64 * 1024, 1 << 20];

/// Group-name filter from `ZENHASH_BENCH`.
fn wanted(name: &str) -> bool {
    std::env::var("ZENHASH_BENCH").map_or(true, |f| name.contains(&f))
}

fn xxh3_64(suite: &mut Suite) {
    for len in SIZES {
        let input = data(len);
        let name = format!("xxh3_64/{len}");
        if !wanted(&name) {
            continue;
        }
        suite.group(name, |g| {
            g.throughput(Throughput::Bytes(len as u64));
            g.bench("zenhash", move |b| {
                b.iter(|| zenhash::xxh3_64(black_box(input)))
            });
            g.bench("twox-hash", move |b| {
                b.iter(|| twox_hash::XxHash3_64::oneshot(black_box(input)))
            });
            g.bench("xxhash-rust", move |b| {
                b.iter(|| xxhash_rust::xxh3::xxh3_64(black_box(input)))
            });
        });
    }
}

fn xxh3_128(suite: &mut Suite) {
    for len in [16usize, 240, 64 * 1024] {
        let input = data(len);
        let name = format!("xxh3_128/{len}");
        if !wanted(&name) {
            continue;
        }
        suite.group(name, |g| {
            g.throughput(Throughput::Bytes(len as u64));
            g.bench("zenhash", move |b| {
                b.iter(|| zenhash::xxh3_128(black_box(input)))
            });
            g.bench("twox-hash", move |b| {
                b.iter(|| twox_hash::XxHash3_128::oneshot(black_box(input)))
            });
            g.bench("xxhash-rust", move |b| {
                b.iter(|| xxhash_rust::xxh3::xxh3_128(black_box(input)))
            });
        });
    }
}

fn xxh3_streaming(suite: &mut Suite) {
    let len = 1 << 20;
    let input = data(len);
    for chunk in [64usize, 4096] {
        let name = format!("xxh3_64_stream/1MiB_in_{chunk}B_chunks");
        if !wanted(&name) {
            continue;
        }
        suite.group(name, move |g| {
            g.throughput(Throughput::Bytes(len as u64));
            g.bench("zenhash", move |b| {
                b.iter(|| {
                    let mut h = zenhash::Xxh3::new();
                    for c in input.chunks(chunk) {
                        h.update(c);
                    }
                    h.digest()
                })
            });
            g.bench("twox-hash", move |b| {
                b.iter(|| {
                    let mut h = twox_hash::XxHash3_64::new();
                    for c in input.chunks(chunk) {
                        h.write(c);
                    }
                    h.finish()
                })
            });
            g.bench("xxhash-rust", move |b| {
                b.iter(|| {
                    let mut h = xxhash_rust::xxh3::Xxh3::new();
                    for c in input.chunks(chunk) {
                        h.update(c);
                    }
                    h.digest()
                })
            });
        });
    }
}

fn xxh64_xxh32(suite: &mut Suite) {
    for len in [16usize, 64 * 1024] {
        let input = data(len);
        let name = format!("xxh64/{len}");
        if wanted(&name) {
            suite.group(name, |g| {
                g.throughput(Throughput::Bytes(len as u64));
                g.bench("zenhash", move |b| {
                    b.iter(|| zenhash::xxh64(black_box(input), 0))
                });
                g.bench("twox-hash", move |b| {
                    b.iter(|| twox_hash::XxHash64::oneshot(0, black_box(input)))
                });
                g.bench("xxhash-rust", move |b| {
                    b.iter(|| xxhash_rust::xxh64::xxh64(black_box(input), 0))
                });
            });
        }
        let name = format!("xxh32/{len}");
        if !wanted(&name) {
            continue;
        }
        suite.group(name, |g| {
            g.throughput(Throughput::Bytes(len as u64));
            g.bench("zenhash", move |b| {
                b.iter(|| zenhash::xxh32(black_box(input), 0))
            });
            g.bench("twox-hash", move |b| {
                b.iter(|| twox_hash::XxHash32::oneshot(0, black_box(input)))
            });
            g.bench("xxhash-rust", move |b| {
                b.iter(|| xxhash_rust::xxh32::xxh32(black_box(input), 0))
            });
        });
    }
}

zenbench::main!(xxh3_64, xxh3_128, xxh3_streaming, xxh64_xxh32);
