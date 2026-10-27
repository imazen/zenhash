//! Standard rapidhash v3 vs the existing Rust port, with identical default secrets.
use zenbench::prelude::*;

fn rapid(suite: &mut Suite) {
    for len in [4, 16, 64, 112, 113, 240, 1024, 65536] {
        let data: &'static [u8] = (0..len).map(|i| (i * 7) as u8).collect::<Vec<_>>().leak();
        suite.group(format!("rapidhash_v3/{len}"), |g| {
            g.throughput(Throughput::Bytes(len as u64));
            g.bench("zenhash", move |b| {
                b.iter(|| zenhash::rapidhash_v3(black_box(data)))
            });
            g.bench("rapidhash", move |b| {
                b.iter(|| rapidhash::v3::rapidhash_v3(black_box(data)))
            });
        });
    }
    let data = vec![0x57; 65536].leak();
    for chunk in [64, 4096] {
        let data: &'static [u8] = data;
        suite.group(format!("rapidhash_v3_stream/{chunk}"), |g| {
            g.throughput(Throughput::Bytes(data.len() as u64));
            g.bench("zenhash", move |b| {
                b.iter(|| {
                    let mut h = zenhash::RapidhashV3::new();
                    for bytes in black_box(data).chunks(chunk) {
                        h.update(bytes);
                    }
                    h.digest()
                })
            });
            g.bench("rapidhash", move |b| {
                b.iter(|| {
                    let mut h = rapidhash::v3::RapidStreamHasherV3::new(
                        &rapidhash::v3::DEFAULT_RAPID_SECRETS,
                    );
                    for bytes in black_box(data).chunks(chunk) {
                        h.write(bytes);
                    }
                    h.finish()
                })
            });
        });
    }
}

zenbench::main!(rapid);
