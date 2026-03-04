import pytest
import pickle

from perfect_hash import generate_hash

keys = "Je suis un ensemble de clés".split()
many_keys = ["aabb" + hex(i) for i in range(2048 * 8)]

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


def test_custom_values():
    values = [3 * k + 9 for k in range(len(keys))]
    for _ in range(32):
        h = generate_hash(keys, values=values)
        for key, value in zip(keys, values):
            assert h[key] == value


def test_duplicate_values():
    values = [0 for _ in keys]
    h = generate_hash(keys, values=values)
    for key in keys:
        assert h[key] == 0


def test_python_object_values():
    values = [{"key": key} for key in keys]
    h = generate_hash(keys, values=values)
    for key, value in zip(keys, values):
        assert h[key] is value


def test_creation(benchmark):
    benchmark.pedantic(generate_hash, args=(many_keys, ), rounds=10)  


def test_many():
    h = generate_hash(many_keys)
    for k, v in enumerate(many_keys):
        assert h[v] == k


def test_pickle():
    h = generate_hash(many_keys)
    dump = pickle.dumps(h)
    h = pickle.loads(dump)
    for k, v in enumerate(many_keys):
        assert h[v] == k


def test_missing_key_raises_key_error():
    h = generate_hash(keys)
    with pytest.raises(KeyError):
        h["not present"]


def test_missing_key_raises_key_error_after_pickle():
    h = generate_hash(keys)
    dump = pickle.dumps(h)
    h = pickle.loads(dump)
    with pytest.raises(KeyError):
        h["still not present"]
