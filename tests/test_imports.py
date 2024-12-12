import pytest

from perfect_hash import generate_hash

keys = "Je suis un ensemble de clés".split()


def test_creation():
    # with pytest.raises(Exception):
    #     pass

    generate_hash(keys)