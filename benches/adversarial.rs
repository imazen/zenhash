//! Adversarial comparison: every competitor in its most favorable setup.
//!
//! Run through `just bench-adversarial`, which builds with
//! `-C target-cpu=native` (xxhash-rust picks its SIMD at compile time, so this
//! hands it AVX2/AVX-512), `CFLAGS=-O3 -march=native` for the C reference,
//! fat LTO and one codegen unit. Each crate uses its fastest API for the job.
//! `just bench-adversarial-shipped` keeps the same competitor setup but builds
//! zenhash the way users get it (runtime dispatch, no native flags; CFLAGS
//! still apply to C).
//!
//! Beyond fixed sizes, the `keys_*` groups hash batches of mixed-length keys
//! in shuffled order, which defeats branch prediction on the size dispatch.
//!
//! `ZENHASH_BENCH=<substring>` filters groups.
use std::hash::Hasher;

use zenbench::prelude::*;

fn wanted(name: &str) -> bool {
    std::env::var("ZENHASH_BENCH").map_or(true, |f| name.contains(&f))
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// Bench closures must be `'static`; leaking a few MiB once per run is fine.
fn data(len: usize, seed: u64) -> &'static [u8] {
    let mut rng = Rng(seed);
    (0..len)
        .map(|_| rng.next() as u8)
        .collect::<Vec<u8>>()
        .leak()
}

const SIZES: [usize; 13] = [
    8,
    16,
    32,
    64,
    128,
    129,
    240,
    241,
    512,
    1024,
    4096,
    64 * 1024,
    1 << 20,
];

fn xxh3_64(suite: &mut Suite) {
    for len in SIZES {
        let name = format!("xxh3_64/{len}");
        if !wanted(&name) {
            continue;
        }
        let input = data(len, len as u64);
        suite.group(name, move |g| {
            g.throughput(Throughput::Bytes(len as u64));
            g.bench("zenhash", move |b| {
                b.iter(|| zenhash::xxh3_64(black_box(input)))
            });
            g.bench("C xxHash 0.8.3", move |b| {
                b.iter(|| xxhash_c::xxh3_64(black_box(input)))
            });
            g.bench("xxhash-rust", move |b| {
                b.iter(|| xxhash_rust::xxh3::xxh3_64(black_box(input)))
            });
            g.bench("twox-hash", move |b| {
                b.iter(|| twox_hash::XxHash3_64::oneshot(black_box(input)))
            });
        });
    }
}

fn xxh3_128(suite: &mut Suite) {
    for len in [16usize, 240, 1024, 64 * 1024] {
        let name = format!("xxh3_128/{len}");
        if !wanted(&name) {
            continue;
        }
        let input = data(len, len as u64);
        suite.group(name, move |g| {
            g.throughput(Throughput::Bytes(len as u64));
            g.bench("zenhash", move |b| {
                b.iter(|| zenhash::xxh3_128(black_box(input)))
            });
            g.bench("C xxHash 0.8.3", move |b| {
                b.iter(|| xxhash_c::xxh3_128(black_box(input)))
            });
            g.bench("xxhash-rust", move |b| {
                b.iter(|| xxhash_rust::xxh3::xxh3_128(black_box(input)))
            });
            g.bench("twox-hash", move |b| {
                b.iter(|| twox_hash::XxHash3_128::oneshot(black_box(input)))
            });
        });
    }
}

fn seeded(suite: &mut Suite) {
    const SEED: u64 = 0x9E37_79B9_7F4A_7C15;
    for len in [16usize, 240, 64 * 1024] {
        let name = format!("xxh3_64_seeded/{len}");
        if !wanted(&name) {
            continue;
        }
        let input = data(len, len as u64);
        // xxhash-c has no seeded one-shot; its streaming state is the C
        // library's only seeded entry point, so it's left out here.
        suite.group(name, move |g| {
            g.throughput(Throughput::Bytes(len as u64));
            g.bench("zenhash", move |b| {
                b.iter(|| zenhash::xxh3_64_with_seed(black_box(input), black_box(SEED)))
            });
            g.bench("xxhash-rust", move |b| {
                b.iter(|| xxhash_rust::xxh3::xxh3_64_with_seed(black_box(input), black_box(SEED)))
            });
            g.bench("twox-hash", move |b| {
                b.iter(|| {
                    twox_hash::XxHash3_64::oneshot_with_seed(black_box(SEED), black_box(input))
                })
            });
        });
    }
}

/// Batches of keys with lengths drawn uniformly from `1..=max`, in shuffled
/// order. Throughput counts key bytes.
fn keys(suite: &mut Suite) {
    for (max, count) in [(16usize, 4096usize), (64, 4096), (512, 1024)] {
        let name = format!("keys_mixed_1to{max}");
        if !wanted(&name) {
            continue;
        }
        let mut rng = Rng(max as u64);
        let blob = data(count * max, 7);
        let keys: &'static [&'static [u8]] = (0..count)
            .map(|i| {
                let len = 1 + (rng.next() % max as u64) as usize;
                &blob[i * max..i * max + len]
            })
            .collect::<Vec<_>>()
            .leak();
        let bytes: u64 = keys.iter().map(|k| k.len() as u64).sum();
        suite.group(name, move |g| {
            g.throughput(Throughput::Bytes(bytes));
            g.bench("zenhash", move |b| {
                b.iter(|| {
                    keys.iter()
                        .fold(0u64, |a, k| a ^ zenhash::xxh3_64(black_box(k)))
                })
            });
            g.bench("C xxHash 0.8.3", move |b| {
                b.iter(|| {
                    keys.iter()
                        .fold(0u64, |a, k| a ^ xxhash_c::xxh3_64(black_box(k)))
                })
            });
            g.bench("xxhash-rust", move |b| {
                b.iter(|| {
                    keys.iter()
                        .fold(0u64, |a, k| a ^ xxhash_rust::xxh3::xxh3_64(black_box(k)))
                })
            });
            g.bench("twox-hash", move |b| {
                b.iter(|| {
                    keys.iter().fold(0u64, |a, k| {
                        a ^ twox_hash::XxHash3_64::oneshot(black_box(k))
                    })
                })
            });
        });
    }
}

fn streaming(suite: &mut Suite) {
    let len = 1 << 20;
    let input = data(len, 99);
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
            g.bench("C xxHash 0.8.3", move |b| {
                b.iter(|| {
                    let mut h = xxhash_c::XXH3_64::new();
                    for c in input.chunks(chunk) {
                        h.write(c);
                    }
                    h.finish()
                })
            });
            // Xxh3Default: xxhash-rust's streaming type specialized for the
            // default secret, its fastest streaming option.
            g.bench("xxhash-rust", move |b| {
                b.iter(|| {
                    let mut h = xxhash_rust::xxh3::Xxh3Default::new();
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
        });
    }
}

fn classic(suite: &mut Suite) {
    for len in [16usize, 64 * 1024] {
        let input = data(len, 3);
        let name = format!("xxh64/{len}");
        if wanted(&name) {
            suite.group(name, move |g| {
                g.throughput(Throughput::Bytes(len as u64));
                g.bench("zenhash", move |b| {
                    b.iter(|| zenhash::xxh64(black_box(input), 0))
                });
                g.bench("C xxHash 0.8.3", move |b| {
                    b.iter(|| xxhash_c::xxh64(black_box(input), 0))
                });
                g.bench("xxhash-rust", move |b| {
                    b.iter(|| xxhash_rust::xxh64::xxh64(black_box(input), 0))
                });
                g.bench("twox-hash", move |b| {
                    b.iter(|| twox_hash::XxHash64::oneshot(0, black_box(input)))
                });
            });
        }
        let name = format!("xxh32/{len}");
        if wanted(&name) {
            suite.group(name, move |g| {
                g.throughput(Throughput::Bytes(len as u64));
                g.bench("zenhash", move |b| {
                    b.iter(|| zenhash::xxh32(black_box(input), 0))
                });
                g.bench("C xxHash 0.8.3", move |b| {
                    b.iter(|| xxhash_c::xxh32(black_box(input), 0))
                });
                g.bench("xxhash-rust", move |b| {
                    b.iter(|| xxhash_rust::xxh32::xxh32(black_box(input), 0))
                });
                g.bench("twox-hash", move |b| {
                    b.iter(|| twox_hash::XxHash32::oneshot(0, black_box(input)))
                });
            });
        }
    }
}

zenbench::main!(xxh3_64, xxh3_128, seeded, keys, streaming, classic);
