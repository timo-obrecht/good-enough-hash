from typing import Any, Callable, Optional

from perfect_hash._core import generate_hasher, from_args


class Hash[T]:

    # make this thing picklable
    # https://docs.python.org/3/library/pickle.html#what-can-be-pickled-and-unpickled

    def __init__(self, hasher):
        self._hasher = hasher

    def __call__(self, item: str):
        return self._hasher(item)
    
    def __getitem__(self, item: str) -> T:
        return self._hasher(item)


    # those methods are to make the object pickable
    def __getstate__(self):
        state = self._hasher.dump()
        keywords = (
            "tag_seed",
            "bucket_seed",
            "bucket_count",
            "table_len",
            "pilots",
            "values",
            "tags",
        )
        return {k : s for k, s in zip(keywords, state)}


    def __setstate__(self, state):
        self._hasher = from_args(
            state["tag_seed"],
            state["bucket_seed"],
            state["bucket_count"],
            state["table_len"],
            state["pilots"],
            state["values"],
            state["tags"],
        )


def generate_hash(
    
        keys: list[str] | tuple[str],
        values: Optional[list[T]] = None,
        ) -> Callable[[str], T]:
    """Generate a perfect hash function for a set of keys.

    Args:
        keys: A list or tuple of strings to hash.

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

    _keys = list(keys)

    if values is not None:
        _values = list(values)
        if len(_values) != len(_keys):
            raise ValueError("values must have the same length as keys.")
    else:
        _values = list(range(len(_keys)))

    # Generate the perfect hash function.
    h = generate_hasher(_keys, _values)
    return Hash(h)
