from __future__ import annotations


def generate_hasher(keys: list[str], values: list[int]): ...
def from_args(
    ng: int,
    seed1: int,
    seed2: int,
    indices: list[int],
    values: list[int],
    key_tags: list[int],
    key_offsets: list[int],
    key_data: bytes,
): ...
