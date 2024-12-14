from perfect_hash.generation import generate_hash

def main() -> None:
    keys = "Je suis un ensemble de clés".split()
    values = [3*k + 9 for k in range(len(keys))]
    h = generate_hash(keys, values=values)
    for k, v in enumerate(keys):
        print(f"{k} || {v} : {h(v)}")



if __name__ == "__main__":
    main()