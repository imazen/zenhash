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

# Same, with the AVX-512 tier compiled in
bench-avx512 *ARGS:
    cargo bench --bench hash --features avx512 {{ARGS}}

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
