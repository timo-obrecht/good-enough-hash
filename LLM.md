# LLM.md

## Purpose

This repository is an experimental Python package backed by a Rust extension that builds a static hash mapper for a fixed set of unique strings.

The public API is small:

- `perfect_hash.generate_hash(keys, values=None) -> Hash`
- the returned `Hash` object supports both `h(key)` and `h[key]`
- the object is designed to be picklable

This is not a generic mutable dictionary. It is a precomputed mapping for a known set of keys.

## Architecture

### Python Layer

- Package root: `src/perfect_hash`
- Public export: `src/perfect_hash/__init__.py`
- Wrapper logic: `src/perfect_hash/generation.py`

`generate_hash()` does the Python-side validation:

- all keys must be `str`
- all keys must be unique
- default values are `0..len(keys)-1` if `values` is not provided

The Python `Hash` wrapper:

- delegates lookups to the Rust object
- implements `__call__` and `__getitem__`
- implements `__getstate__` / `__setstate__` for pickle support

If you change the Rust `dump()` / `from_args()` contract, you must update the Python pickle logic with it.

### Rust Layer

- Extension source: `src/lib.rs`
- Built as the Python module `perfect_hash._core`

Core responsibilities in Rust:

- build a graph from two randomized base hashes
- retry until an acyclic assignment is found
- store per-vertex values used to reconstruct the final key index/value
- expose the generated hasher to Python through `pyo3`

Important implementation details:

- the crate uses `#![feature(portable_simd)]`
- hashing currently uses `std::simd`
- the build therefore depends on a nightly Rust toolchain
- the graph search is retry-based and can grow `ng` over time

The FFI surface is intentionally narrow:

- `generate_hasher(keys, values)`
- `from_args(ng, f1, f2, indices)`

If you add or rename exported Rust functions, also update:

- `src/perfect_hash/generation.py`
- `src/perfect_hash/_core.pyi`

## Repository Layout

- `src/lib.rs`: Rust extension implementation
- `src/perfect_hash/generation.py`: Python wrapper and input validation
- `src/perfect_hash/_core.pyi`: minimal type stub for the extension
- `tests/test_hash.py`: functional tests and pickle coverage
- `tests/bench.py`: ad hoc benchmark script
- `README.md`: project notes, performance experiments, and TODOs
- `.github/workflows/buid_wheels.yml`: wheel/sdist build and publish workflow

## Build And Dev Workflow

This project uses `maturin` as the build backend and `uv` is present for dependency management.

Typical local workflow:

1. Install dependencies: `uv sync --dev`
2. Build/install extension in editable mode: `uv run maturin develop --release`
3. Run tests: `uv run pytest`

Useful alternatives:

- `uv build` to build artifacts through the configured backend
- `uv run python tests/bench.py` for the simple benchmark script

Because `src/lib.rs` uses nightly-only features, use a nightly toolchain when building. The GitHub workflow currently pins `nightly-2024-08-19`.

## Current Constraints And Mismatches

These are important when modifying the repo:

- `src/lib.rs` requires nightly Rust because of `portable_simd`.
- `pyproject.toml` says `requires-python = ">=3.12"`, but CI builds wheels for Python 3.10.
- `pyproject.toml` defines a console script `perfect-hash = "perfect_hash:main"`, but there is no `main` function in `src/perfect_hash/__init__.py`.
- `tests/test_hash.py` includes `test_creation(benchmark)`, which expects the `pytest-benchmark` fixture, but that dependency is not listed in the `dev` group.
- `README.md` is partly exploratory and not a reliable source of current packaging behavior.

Do not treat the packaging metadata as fully coherent without checking these points first.

## Testing Notes

The main test coverage is around:

- deterministic lookup correctness
- larger input sets
- pickle round-tripping

There is a commented-out custom-values test. If you touch `values` handling, restore or replace that coverage.

Be careful with `generate_hash(..., values=...)`:

- the current Python code uses `if values:`
- an empty list is treated the same as `None`
- changes here should be deliberate and tested

## Editing Guidance

When working in this repo:

- keep the Python API small and stable unless the change explicitly expands it
- preserve the pickle format unless a breaking change is intended
- keep Python validation and Rust assumptions aligned
- update both Python and Rust sides together for any FFI change
- prefer benchmarking changes in `tests/bench.py` or dedicated tests instead of relying on intuition

If you touch build, release, or compatibility metadata, check all of:

- `pyproject.toml`
- `Cargo.toml`
- `.github/workflows/buid_wheels.yml`

Those files currently encode overlapping assumptions and are easy to drift out of sync.

## What To Read First

For most code changes, read in this order:

1. `src/perfect_hash/generation.py`
2. `src/lib.rs`
3. `tests/test_hash.py`

That gives the public API, the implementation, and the current expected behavior with minimal context load.
