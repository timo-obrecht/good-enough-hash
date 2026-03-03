# Perfect Hash Implementation Plan

## Goal

Build a static string-to-integer mapper that:

- raises `KeyError` for unknown keys
- stays callable from Python with `generate_hash(keys, values=None)`
- is competitive on lookup speed
- is smaller than a Python `dict` when possible, but correctness comes first

## Benchmark Storage

Benchmark snapshots are stored outside this file:

- CSV: `benchmark_results/latest.csv`
- JSON: `benchmark_results/latest.json`

Regenerate the latest snapshot with:

```bash
uv run python tests/benchmark_matrix.py --lookup-repeats 32 --output-stem latest
```

## Latest Benchmark Summary

Latest run results from `benchmark_results/latest.*`:

| implementation | size | build s | hit s | miss s | RSS delta MB | serialized MB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| dict | 10,000 | 0.001473 | 0.003428 | 0.015436 | 0.41 | 0.25 |
| perfect_hash | 10,000 | 0.025878 | 0.012352 | 0.035567 | 1.19 | 0.34 |
| dict | 50,000 | 0.007898 | 0.003329 | 0.014987 | 3.16 | 1.24 |
| perfect_hash | 50,000 | 0.538473 | 0.021332 | 0.046020 | 5.03 | 1.69 |
| dict | 100,000 | 0.039768 | 0.005760 | 0.015540 | 6.53 | 2.55 |
| perfect_hash | 100,000 | 0.863093 | 0.013715 | 0.049411 | 8.19 | 3.51 |
| dict | 500,000 | 0.269323 | 0.007221 | 0.024207 | 29.78 | 13.23 |
| perfect_hash | 500,000 | 4.097636 | 0.024716 | 0.047661 | 32.19 | 17.99 |
| dict | 1,000,000 | 0.421744 | 0.014510 | 0.030140 | 59.77 | 26.58 |
| perfect_hash | 1,000,000 | 10.703376 | 0.026052 | 0.060308 | 64.57 | 36.21 |

## Current State

- Unknown keys are detected exactly by comparing the queried bytes to the stored key bytes at the computed slot.
- Lookups first use a 1-byte slot tag to reject many misses before the full byte compare.
- The hot path computes both graph hashes in one pass and uses power-of-two table sizes to avoid modulo.
- Custom `values=` is preserved by storing returned values separately from the stable slot index.

## Next Steps

1. Reduce memory in the hot structure by shrinking integer storage (`usize` to `u32` where safe).
2. Re-run the same benchmark matrix and compare `500K` and `1M` memory deltas against the current `latest.*` snapshot.
3. Replace the `HashMap` graph representation only after the lookup and memory layout are in a better place.
