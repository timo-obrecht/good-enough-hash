from typing import Callable

from perfect_hash._core import hello_from_bin
from perfect_hash._core import generate_hasher

def generate_hash(keys: list[str] | set[str]) -> Callable[[str], int]:
    """Generate a perfect hash function for a set of keys.

    Args:
        keys: A list or set of strings to hash.

    Returns:
        A function that hashes a string to an integer.

    Raises:
        ValueError: If the keys are not unique.
        ValueError: If the keys are not strings.
    """
    if not all(isinstance(key, str) for key in keys):
        raise ValueError("All keys must be strings.")
    if len(keys) != len(set(keys)):
        raise ValueError("All keys must be unique.")

    # Generate the perfect hash function.
    return generate_hasher(list(keys))