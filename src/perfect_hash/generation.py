from typing import Callable, Optional

from perfect_hash._core import generate_hasher


class Hash:

    def __init__(self, hasher):
        self._hasher = hasher

    def __call__(self, *args, **kwds):
        return self._hasher.call(args[0])


def generate_hash(
        keys: list[str] | set[str],
        order: Optional[list[int]] = None,
        ) -> Callable[[str], int]:
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

    if order:
        assert set(order) == set(range(len(keys)))
        _keys = [keys[i] for i in order]
    else:
        _keys = list(keys)

    # Generate the perfect hash function.
    h = generate_hasher(_keys)
    return Hash(h)