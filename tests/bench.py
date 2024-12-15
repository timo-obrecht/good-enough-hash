from perfect_hash import generate_hash
from timeit import timeit
import random
import pickle


def main() -> None:
    # keys = "Je suis un ensemble de clés".split()
    # values = [3*k + 9 for k in range(len(keys))]
    # h = generate_hash(keys, values=values)
    # for k, v in enumerate(keys):
    #     print(f"{k} || {v} : {h(v)}")

    # keys = "Je suis un ensemble de clés".split()
    # keys = ['abcd' + hex(i) + 'wxyz' for i in range(2**22)]
    keys = ['abcd' + hex(i) + 'wxyz' for i in range(2**15 - 1987)]

    custom_map = generate_hash(keys)
    # print("finished compute")

    # with open("tmp", "wb") as file:
    #     pickle.dump(custom_map, file)

    # with open("tmp", "rb") as file:
    #     custom_map = pickle.load(file)    

    default_map = {k : v for v, k in enumerate(keys)}

    some_keys = random.sample(keys, 4096)

    a = timeit(lambda: [custom_map[key] for key in some_keys], number=10000)
    b = timeit(lambda: [default_map[key] for key in some_keys], number=10000)

    print(f"perfect hash : {a}")
    print(f"not perfect hash : {b}")



if __name__ == "__main__":
    main()