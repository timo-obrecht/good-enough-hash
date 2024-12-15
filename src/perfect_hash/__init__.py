from perfect_hash.generation import generate_hash
from timeit import timeit
import random
import pickle


def main() -> None:
    # keys = "Je suis un ensemble de clés".split()
    # values = [3*k + 9 for k in range(len(keys))]
    # h = generate_hash(keys, values=values)
    # for k, v in enumerate(keys):
    #     print(f"{k} || {v} : {h(v)}")

    keys = "Je suis un ensemble de clés".split()
    many_keys = ['aakk' + hex(i) for i in range(2048 * 8)]

    keys = many_keys

    h = generate_hash(keys)

    with open("tmp", "wb") as file:
        pickle.dump(h, file)

    with open("tmp", "rb") as file:
        h = pickle.load(file)    

    print("finished compute")
    normal_map = {k : v for v, k in enumerate(keys)}

    some_keys = random.sample(keys, 1000)

    a = timeit(lambda: [h[key] for key in some_keys], number=10000)
    b = timeit(lambda: [normal_map[key] for key in some_keys], number=10000)

    print(f"perfect hash : {a}")
    print(f"not perfect hash : {b}")



if __name__ == "__main__":
    main()