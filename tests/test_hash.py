import pytest

from perfect_hash import generate_hash

keys = "Je suis un ensemble de clés".split()

many_keys = [hex(i) for i in range(2048 * 8)]

def test_call():
    for _ in range(32):
        h = generate_hash(keys)
        for k, v in enumerate(keys):
            assert h(v) == k


def test_values():
    for _ in range(32):
        h = generate_hash(keys)
        for k, v in enumerate(keys):
            assert h[v] == k


# def test_custom_values():
#     values = [3*k + 9 for k in range(len(keys))]
#     for _ in range(32):
#         h = generate_hash(keys, values=values)
#         for k, v in zip(keys, values):
#             assert h[k] == v


def test_many():
    for _ in range(4):
        h = generate_hash(many_keys)
        for k, v in enumerate(many_keys):
            assert h[v] == k
