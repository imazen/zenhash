#![forbid(unsafe_code)]
fn main() {
    std::hint::black_box(zenhash_wasm_workload::hash_batch(0, 64, 7, 0, 1));
}
