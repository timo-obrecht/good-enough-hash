from __future__ import annotations

from typing import Any


def generate_hasher(keys: list[str], values: list[Any]): ...
def from_args(
    tag_seed: int,
    bucket_seed: int,
    bucket_count: int,
    table_len: int,
    pilots: list[int],
    values: list[Any],
    tags: list[int],
): ...
