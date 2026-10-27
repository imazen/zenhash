# Rapidhash v3 baseline, 2026-10-26

Source: `16ab32136e74b56d3c458c87a725ac6475af4b98`.
Benchmark: `benches/rapid.rs`, SHA-256 `786191192794e4f484563366a9209df487fc05e63bcaf9836dcfba00f59505d6`.
CPU: AMD Ryzen 9 7950X; WSL2 Linux 6.18.40.1; rustc 1.99.1.
Compared with rapidhash 4.5.1, standard v3 and identical default secrets.
No `target-cpu=native`; default Cargo bench profile; zenbench 0.1.11.

Command: `TMPDIR=$HOME/tmp ~/work/claudehints/scripts/run-heavy --jobs 4 -- just bench-rapid -- --no-busy-gate --format=md`

One exploratory run, not a cross-ISA performance claim. Small one-shot inputs
and 4 KiB streaming updates have gaps; ARM and WASM throughput was not measured.
The comparison intervals below are zenbench's round-level estimates of the
competitor's time relative to zenhash; use the full evidence rather than ratios
computed from rounded means.

# Benchmark Results

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=null cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

**git:** `16ab32136e74b56d3c458c87a725ac6475af4b98`  
**total:** 7.1s  **waits:** 0 (0.0s)

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3/4

*30 rounds × 500000 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 1.7ns | 2.4ns | 2.4ns | 1.58 GiB/s |
| rapidhash | 0.8ns | 1.1ns | [-58.6%  -55.0%  -51.5%] | 3.52 GiB/s |

```
  zenhash    |█████████████▍                | 1.6 GiB/s
  rapidhash  |██████████████████████████████| 3.5 GiB/s
```

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3/16

*30 rounds × 500000 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 1.7ns | 2.1ns | 2.1ns | 7.02 GiB/s |
| rapidhash | 0.8ns | 1.0ns | [-56.3%  -53.3%  -50.6%] | 15.1 GiB/s |

```
  zenhash    |█████████████▉                |  7.0 GiB/s
  rapidhash  |██████████████████████████████| 15.1 GiB/s
```

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3/64

*30 rounds × 500000 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 2.5ns | 2.6ns | 2.6ns | 23.0 GiB/s |
| rapidhash | 2.7ns | 2.8ns | [+5.5%  +7.6%  +9.7%] | 21.4 GiB/s |

```
  zenhash    |██████████████████████████████| 23.0 GiB/s
  rapidhash  |███████████████████████████▉  | 21.4 GiB/s
```

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3/112

*30 rounds × 333333 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 3.3ns | 3.4ns | 3.4ns | 30.5 GiB/s |
| rapidhash | 3.4ns | 3.5ns | [+0.5%  +2.1%  +3.7%] | 29.9 GiB/s |

```
  zenhash    |██████████████████████████████| 30.5 GiB/s
  rapidhash  |█████████████████████████████▍| 29.9 GiB/s
```

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3/113

*30 rounds × 250000 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 4.3ns | 4.4ns | 4.4ns | 23.7 GiB/s |
| rapidhash | 4.5ns | 4.7ns | [+4.0%  +5.2%  +6.7%] | 22.2 GiB/s |

```
  zenhash    |██████████████████████████████| 23.7 GiB/s
  rapidhash  |████████████████████████████▏ | 22.2 GiB/s
```

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3/240

*30 rounds × 111111 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 6.7ns | 7.7ns | 7.7ns | 28.9 GiB/s |
| rapidhash | 6.7ns | 8.1ns | [-0.1%  +3.9%  +7.9%] | 27.5 GiB/s |

```
  zenhash    |██████████████████████████████| 28.9 GiB/s
  rapidhash  |████████████████████████████▌ | 27.5 GiB/s
```

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3/1024

*30 rounds × 37037 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 23.8ns | 24.1ns | 24.1ns | 39.5 GiB/s |
| rapidhash | 25.4ns | 26.1ns | [+7.4%  +8.1%  +8.9%] | 36.5 GiB/s |

```
  zenhash    |██████████████████████████████| 39.5 GiB/s
  rapidhash  |███████████████████████████▋  | 36.5 GiB/s
```

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3/65536

*30 rounds × 612 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 1.49µs | 1.54µs | 1.54µs | 39.7 GiB/s |
| rapidhash | 1.59µs | 1.65µs | [+5.7%  +6.8%  +7.8%] | 37.1 GiB/s |

```
  zenhash    |██████████████████████████████| 39.7 GiB/s
  rapidhash  |███████████████████████████▉  | 37.1 GiB/s
```

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3_stream/64

*30 rounds × 129 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 5.79µs | 7.22µs | 7.22µs | 8.46 GiB/s |
| rapidhash | 5.91µs | 6.86µs | [-2.3%  +0.7%  +3.7%] | 8.90 GiB/s |

```
  zenhash    |████████████████████████████▌ | 8.5 GiB/s
  rapidhash  |██████████████████████████████| 8.9 GiB/s
```

CPU placement: `mode=iterate inference=exploratory cpu_policy=auto cpu_placement=uniform_hardware cpu_class="capacity1024_l332768K" cpu_enforcement=cpuset cpu_effective_set=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_requested_set=null cpu_thread_requested=null cpu_thread_effective=[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31] cpu_class_uniform=true pool_thread_count=unknown cpu_frequency_verified=false`

## rapidhash_v3_stream/4096

*30 rounds × 429 calls*

| Benchmark | Min | Mean | vs Base | Throughput |
|-----------|-----|------|---------|------------|
| zenhash | 2.30µs | 2.40µs | 2.40µs | 25.4 GiB/s |
| rapidhash | 1.75µs | 1.79µs | [-25.1%  -24.3%  -23.6%] | 34.2 GiB/s |

```
  zenhash    |██████████████████████▎       | 25.4 GiB/s
  rapidhash  |██████████████████████████████| 34.2 GiB/s
```

run-heavy: done rc=0 29s | peak-RSS 0.46GiB | min-avail 9270MiB | peak-load 4.41
