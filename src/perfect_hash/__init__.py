from perfect_hash._core import hello_from_bin
from perfect_hash.generation import generate_hash

def main() -> None:
    keys = "Je suis un ensemble de clés pas très long, mais quand même peu".split()
    h = generate_hash(keys)
    # h["Je"]
    for k, v in enumerate(keys):
        print(f"{k} || {v} : {h(v)}")



if __name__ == "__main__":
    main()