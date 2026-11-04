# Rapidhash v3 after separating short and long paths, 2026-10-26

Source: `a86457a177cfd57698a92a75fc3557d2ada2bcf3`.
Benchmark: `benches/rapid.rs`, SHA-256 `786191192794e4f484563366a9209df487fc05e63bcaf9836dcfba00f59505d6`.
CPU: AMD Ryzen 9 7950X; WSL2 Linux 6.18.40.1; rustc 1.99.1.
Compared with rapidhash 4.5.1, standard v3 and identical default secrets.
No `target-cpu=native`; default Cargo bench profile; zenbench 0.1.11.

Command: `just bench-rapid -- --no-busy-gate --format=md` (resource wrapper, four build jobs).

The [baseline](rapidhash_v3_2026-10-26.md) and this run measure both implementations
in interleaved groups. Perf sampling put the short-call work in the hashing closure;
its assembly saved six registers before selecting the input length. Keeping the
seven-lane state out of short callers removed those saves from the short path.
The default-secret specialization preserves constant secret folding for long input.

Both final 4-byte means round to 0.8 ns. The reported relative interval
(+0.1% to +2.7% competitor time) slightly favors zenhash.
At 64 KiB the measured throughputs are 40.2 and 38.0 GiB/s. Streaming 64 KiB with
4 KiB updates remains slower (25.0 versus 32.4 GiB/s). Moving previous-tail capture
after the block loop did not close that gap and was not retained. This is one x86
machine's exploratory result. ARM and WASM throughput remain unmeasured.

# Benchmark Results

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=null cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

**git:** `a86457a177cfd57698a92a75fc3557d2ada2bcf3`
**total:** 6.5s  **waits:** 0 (0.0s)

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3/4

*30 rounds × 1000000 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 0.8ns | 0.8ns | 0.8ns | 4.76 GiB/s |
| rapidhash | 0.8ns | 0.8ns | [+0.1%  +1.3%  +2.7%] | 4.70 GiB/s |

```
  zenhash    |██████████████████████████████| 4.8 GiB/s
  rapidhash  |█████████████████████████████▌| 4.7 GiB/s
```

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3/16

*30 rounds × 1000000 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 0.7ns | 0.8ns | 0.8ns | 18.4 GiB/s |
| rapidhash | 0.8ns | 0.9ns | [+6.6%  +9.8%  +13.1%] | 16.7 GiB/s |

```
  zenhash    |██████████████████████████████| 18.4 GiB/s
  rapidhash  |███████████████████████████▎  | 16.7 GiB/s
```

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3/64

*30 rounds × 333333 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 2.7ns | 3.0ns | 3.0ns | 19.8 GiB/s |
| rapidhash | 2.7ns | 3.0ns | [-2.3%  +0.4%  +3.2%] | 19.7 GiB/s |

```
  zenhash    |██████████████████████████████| 19.8 GiB/s
  rapidhash  |█████████████████████████████▉| 19.7 GiB/s
```

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3/112

*30 rounds × 333333 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 3.8ns | 4.0ns | 4.0ns | 26.1 GiB/s |
| rapidhash | 3.3ns | 3.5ns | [-14.9%  -13.1%  -11.6%] | 29.6 GiB/s |

```
  zenhash    |██████████████████████████▌   | 26.1 GiB/s
  rapidhash  |██████████████████████████████| 29.6 GiB/s
```

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3/113

*30 rounds × 200000 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 4.5ns | 5.2ns | 5.2ns | 20.4 GiB/s |
| rapidhash | 4.5ns | 5.1ns | [-3.9%  -1.2%  +1.4%] | 20.7 GiB/s |

```
  zenhash    |█████████████████████████████▋| 20.4 GiB/s
  rapidhash  |██████████████████████████████| 20.7 GiB/s
```

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3/240

*30 rounds × 142857 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 6.8ns | 7.5ns | 7.5ns | 29.8 GiB/s |
| rapidhash | 6.6ns | 7.3ns | [-5.1%  -2.2%  +0.8%] | 30.4 GiB/s |

```
  zenhash    |█████████████████████████████▎| 29.8 GiB/s
  rapidhash  |██████████████████████████████| 30.4 GiB/s
```

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3/1024

*30 rounds × 35714 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 24.7ns | 25.5ns | 25.5ns | 37.4 GiB/s |
| rapidhash | 25.3ns | 26.3ns | [+2.5%  +3.3%  +4.2%] | 36.3 GiB/s |

```
  zenhash    |██████████████████████████████| 37.4 GiB/s
  rapidhash  |█████████████████████████████ | 36.3 GiB/s
```

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3/65536

*30 rounds × 631 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 1.49µs | 1.52µs | 1.52µs | 40.2 GiB/s |
| rapidhash | 1.57µs | 1.61µs | [+4.3%  +5.8%  +7.2%] | 38.0 GiB/s |

```
  zenhash    |██████████████████████████████| 40.2 GiB/s
  rapidhash  |████████████████████████████▎ | 38.0 GiB/s
```

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3_stream/64

*30 rounds × 161 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 5.88µs | 6.03µs | 6.03µs | 10.1 GiB/s |
| rapidhash | 5.81µs | 5.97µs | [-1.7%  -0.9%  -0.2%] | 10.2 GiB/s |

```
  zenhash    |█████████████████████████████▋| 10.1 GiB/s
  rapidhash  |██████████████████████████████| 10.2 GiB/s
```

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3_stream/4096

*30 rounds × 423 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 2.29µs | 2.45µs | 2.45µs | 25.0 GiB/s |
| rapidhash | 1.75µs | 1.88µs | [-24.9%  -23.6%  -22.2%] | 32.4 GiB/s |

```
  zenhash    |███████████████████████       | 25.0 GiB/s
  rapidhash  |██████████████████████████████| 32.4 GiB/s
```

run-heavy: done rc=0 8s | peak-RSS 0.17GiB | min-avail 9421MiB | peak-load 2.03
