from perfect_hash._core import hello_from_bin
from perfect_hash.generation import generate_hash

def main() -> None:
    keys = "Je suis un ensemble de clés".split()
    print(generate_hash(keys))