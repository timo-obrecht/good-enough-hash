from typing import Callable, Optional

from perfect_hash._core import generate_hasher, from_args


class Hash:

    # make this thing picklable
    # https://docs.python.org/3/library/pickle.html#what-can-be-pickled-and-unpickled

    def __init__(self, hasher):
        self._hasher = hasher

    def __call__(self, item: str):
        return self._hasher.call(item)
    
    def __getitem__(self, item: str) -> int:
        return self._hasher.call(item)


    # those methods are to make the object pickable
    def __getstate__(self):
        state = self._hasher.dump()
        keywords = ('ng', 'f1', 'f2', 'indices')
        return {k : s for k, s in zip(keywords, state)}


    def __setstate__(self, state):
        self._hasher = from_args(state['ng'], state['f1'], state['f2'], state['indices'])


def generate_hash(
    
        keys: list[str] | tuple[str],
        values: Optional[list[int]] = None,
        ) -> Callable[[str], int]:
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

    if values:
        s = set(values)
        assert len(values) == len(s)
        values = list(values)
    else:
        values = list(range(len(_keys)))

    # Generate the perfect hash function.
    h = generate_hasher(_keys, values)
    return Hash(h)