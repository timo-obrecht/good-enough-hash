Good Enough Hash
=========


## What it is

*to be written*

## Build

I need to dive a bit more in `uv`. As of right now I am not convinced it is better than pip, but I admit I am low on the learning curve.

`uv build`

publish with `uv publish` ? Or use maturin github actions ?


## Performance


The idea is to have a hash function that is more memory-efficient and faster than a default python dictionnary.

```python
keys = ['abcd' + hex(i) + 'wxyz' for i in range(2048 * 8)]

custom_map = generate_hash(keys)
default_map = {k : v for v, k in enumerate(keys)}

some_keys = random.sample(keys, 1000)

a = timeit(lambda: [custom_map[key] for key in some_keys], number=10000)
b = timeit(lambda: [default_map[key] for key in some_keys], number=10000)
```


At the begining, the custom mapper would take 25 seconds to map 10k elements. For context, the default python mapper takes 0.4s. Not exactly a win.

But I managed to cut this number significantly with a few changes. 

First, is simply...not to forget to compile in release mode. This instantly gave a 10x speedup, down to around 2.2 seconds.

Then, I profiled my code and realized there was a lot of time wasted in PyO3 related stuff. I realized I was actually allocating a new string at every call to the hash function. What I wanted was to read raw bytes from python strings. I just had to change a bit the function signature to accomplish this, a pass bellow 2 seconds at 1.4 seconds.

```rust
// first thing I did was relying on PyO3 default castings
// fn call(&self, key: &str) -> usize

fn call(&self, key: Bound<'_, PyString>) -> usize {
    let data = unsafe { key.data().unwrap() };
    let h1 = self.f1.hash(data.as_bytes());
    let h2 = self.f2.hash(data.as_bytes());
    let combined_index = self.indices[h1].wrapping_add(self.indices[h2]);
    combined_index % self.ng
}
```

Adding an `#[inline(always)]` flag to the hash function gave a modest speedup, down to 1.2 seconds.

Then, SIMD. This is when things become interresting. My first SIMD implementation is actually *slower*, up to 1.8 seconds. I checked incompiler explorer, and I am indeed generating vectorized instructions such as **movaps**.

```rust
#[inline]
fn hash(&self, key: &[u8]) -> usize {
    key.iter().zip(self.salt.iter())
        .map(|(a, b)| (*a as usize).overflowing_mul(*b).0)
        .sum::<usize>()
        % self.ng
}
```

```rust
#[inline]
fn hash(&self, key: &[u8]) -> usize {
    // Convert key bytes and salt to slices
    let key_bytes = key.chunks(CHUNK_SIZE);
    let salt = self.salt.chunks_exact(CHUNK_SIZE);

    let hash = key_bytes.zip(salt).fold(0usize, |acc, (a, b)| {
        let a: Simd<usize, CHUNK_SIZE> = Simd::load_or_default(a).cast();
        let b = Simd::from_slice(b);
        let result = a * b; // overflow is the default behavior with SIMD
        acc.wrapping_add(result.reduce_sum())
    });
    hash % self.ng
}
```

## Ideas for the future

I also realized several things. First, I don't have to use the same hash function as [Ilan](http://ilan.schnell-web.net/prog/perfect-hash/algo.html), and I can use much faster XOR instead of multiplications. Also, I can interpret eight `u8` as a `u64`, so I don't need to cast every byte to a `usize`. Maybe I can also tweak the choice of the graph size to get rid of the modulo operation.

With this, I hope to reach performance comparable to the default python map. But with the inherent problem of the algorithm that I have to load **two** elements from the heap, I am not sure I'll ever be able to outperform it.

## When to use this function

As of right now : never. Just use a regular python dictionary.

On the memory consumption side, if you are dealing with millions of strings, there may be an advantage, but I am more than sceptikal (and I haven't measure anything yet)

The purpose of this project if to implement this algorithm I find very clever (I also need to tune the graph size selection to minimize the runtime expectancy), and to learn how to optimise low-level code (just add `--release`, dumbass).