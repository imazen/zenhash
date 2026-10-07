# zenhash dev tasks — `just` lists them.

ZENUTILS := env_var_or_default("ZENUTILS", justfile_directory() / "../zenutils")
QUALITY := ZENUTILS / "quality"

default:
    @just --list

# Tests in the three feature configurations CI runs
test:
    cargo test
    cargo test --no-default-features
    cargo test --all-features

clippy:
    cargo clippy --all-targets --all-features -- -D warnings
    cargo clippy --all-targets --no-default-features -- -D warnings

# Type-check the SIMD tiers of the architecture you're NOT on (rustup target add both first)
clippy-arch:
    cargo clippy --all-targets --all-features --target aarch64-unknown-linux-gnu -- -D warnings
    cargo clippy --all-targets --all-features --target x86_64-unknown-linux-gnu -- -D warnings
    RUSTFLAGS="-Ctarget-feature=+simd128" cargo clippy --lib --no-default-features --target wasm32-unknown-unknown -- -D warnings

# Run the NEON tier under qemu (apt: qemu-user gcc-aarch64-linux-gnu; on Ubuntu
# that package conflicts with gcc-multilib, which the i686 tests need)
test-aarch64-qemu:
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER="qemu-aarch64 -L /usr/aarch64-linux-gnu" \
    cargo test --target aarch64-unknown-linux-gnu --features _dev

# Big-endian (scalar path, byte-swapping reads) under qemu (apt: qemu-user gcc-s390x-linux-gnu)
test-s390x-qemu:
    CARGO_TARGET_S390X_UNKNOWN_LINUX_GNU_LINKER=s390x-linux-gnu-gcc \
    CARGO_TARGET_S390X_UNKNOWN_LINUX_GNU_RUNNER="qemu-s390x -L /usr/s390x-linux-gnu" \
    cargo test --target s390x-unknown-linux-gnu

# no_std and wasm builds
check-targets:
    cargo check --target thumbv7em-none-eabihf --no-default-features
    cargo check --target wasm32-unknown-unknown --no-default-features
    RUSTFLAGS="-Ctarget-feature=+simd128" cargo check --target wasm32-unknown-unknown

# WASI tests under wasmtime, with and without SIMD128
test-wasm:
    CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime --dir={{justfile_directory()}}::{{justfile_directory()}}" cargo test --target wasm32-wasip1 --lib --tests
    RUSTFLAGS="-Ctarget-feature=+simd128" CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime --dir={{justfile_directory()}}::{{justfile_directory()}}" cargo test --target wasm32-wasip1 --lib --tests

msrv:
    cargo hack check --rust-version

features:
    cargo hack check --feature-powerset --no-dev-deps

# Format, then regenerate docs/public-api/ (commit the diff with the code change)
fmt:
    cargo fmt
    cargo test --manifest-path apidoc/Cargo.toml

api-doc:
    cargo test --manifest-path apidoc/Cargo.toml

# Verify the committed API snapshots without rewriting them
api-doc-check:
    ZEN_API_DOC=check cargo test --manifest-path apidoc/Cargo.toml

# What ships to crates.io, and does the packaged copy build
package:
    cargo package --list --allow-dirty
    cargo package --allow-dirty

fuzz-check:
    cd fuzz && cargo check --all-targets

# Hand-run fuzzing (nightly + cargo-fuzz): just fuzz hash_parity 600
fuzz target secs="600":
    cd fuzz && cargo +nightly fuzz run {{target}} -- -max_total_time={{secs}} -rss_limit_mb=2048

# Benchmarks vs twox-hash and xxhash-rust: just bench, or just bench -- --format=md
bench *ARGS:
    cargo bench --bench hash {{ARGS}}

# Same, without the AVX-512 tier (what a CPU without AVX-512 runs)
bench-no-avx512 *ARGS:
    cargo bench --bench hash --no-default-features --features std {{ARGS}}

# Adversarial: every competitor (incl. the C reference) built for this exact
# CPU with fat LTO; zenhash gets the same flags. See benches/adversarial.rs.
bench-adversarial *ARGS:
    RUSTFLAGS="-C target-cpu=native" CFLAGS="-O3 -march=native" \
    CARGO_PROFILE_BENCH_LTO=fat CARGO_PROFILE_BENCH_CODEGEN_UNITS=1 \
    cargo bench --bench adversarial {{ARGS}}

# Same competitor setup, but zenhash as shipped: runtime dispatch only
# (C still gets -march=native through CFLAGS; Rust crates get no native flags).
bench-adversarial-shipped *ARGS:
    CFLAGS="-O3 -march=native" \
    CARGO_PROFILE_BENCH_LTO=fat CARGO_PROFILE_BENCH_CODEGEN_UNITS=1 \
    cargo bench --bench adversarial {{ARGS}}

# Mutation testing (cargo install cargo-mutants); results in mutants.out/
mutants *ARGS:
    cargo mutants --all-features -j 2 {{ARGS}}

# Line/function coverage (cargo install cargo-llvm-cov)
coverage:
    cargo llvm-cov --all-features --summary-only

# Regenerate README.crates.md from README.md (never edit README.crates.md by hand)
readme:
    @z="{{ZENUTILS}}"; [ -f "$z/scripts/gen-readme-crates.sh" ] || z="{{justfile_directory()}}/.quality-kit"; \
    [ -f "$z/scripts/gen-readme-crates.sh" ] || { echo "kit not found — run: just quality-bootstrap"; exit 2; }; \
    sh "$z/scripts/gen-readme-crates.sh" "{{justfile_directory()}}"

# Everything CI checks that runs locally
ci: clippy test check-targets test-wasm msrv features package fuzz-check api-doc-check
    cargo fmt --check

# Advisory quality sweep from imazen/zenutils (fmt, clippy census, API exposure, stale docs, deny, typos)
quality *flags:
    @q="{{QUALITY}}"; [ -x "$q/quality.sh" ] || q="{{justfile_directory()}}/.quality-kit/quality"; \
    [ -x "$q/quality.sh" ] || { echo "kit not found — run: just quality-bootstrap"; exit 2; }; \
    "$q/quality.sh" --root "{{justfile_directory()}}" {{flags}}

quality-quick:
    @just quality --quick

# Clone the kit (zenutils) into .quality-kit/ when there is no ../zenutils checkout
quality-bootstrap:
    @if [ -d "{{ZENUTILS}}/quality" ]; then echo "kit found at {{ZENUTILS}}"; \
    else git clone --quiet https://github.com/imazen/zenutils "{{justfile_directory()}}/.quality-kit" && \
    echo "cloned kit into .quality-kit (gitignored)"; fi
