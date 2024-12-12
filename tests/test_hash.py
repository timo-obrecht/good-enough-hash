import pytest

from perfect_hash import generate_hash

keys = "Je suis un ensemble de clés".split()


def test_creation():
    # with pytest.raises(Exception):
    #     pass

    hash = generate_hash(keys)
    assert hash("Je") == 0
    assert hash("suis") == 1
    assert hash("un") == 2
    assert hash("ensemble") == 3
    assert hash("de") == 4
    assert hash("clés") == 5


def test_values():
    h = generate_hash(keys)
    for k, v in enumerate(keys):
        assert h[v] == k