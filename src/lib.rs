#![feature(portable_simd)]

use pyo3::prelude::*;
use pyo3::types::{PyString, PyList} ;
use rand::Rng;
use std::collections::HashMap;

use std::simd::num::SimdUint;
use std::simd::Simd;

use pyo3::ffi::PyErr_CheckSignals;

const CHUNK_SIZE: usize = 8;

#[macro_export]
#[cfg(not(test))]
macro_rules! python_interupt {
    ($n_iter: expr, $period: expr) => {
        if $n_iter % $period == 0 {
            unsafe {if PyErr_CheckSignals() == -1 {panic!("Keyboard interupt");}}
        }
    };
}

#[macro_export]
#[cfg(test)]
macro_rules! python_interupt {
    ($n_iter: expr, $period: expr) => {};
}


struct Graph {
    n: usize,
    ng: usize, // Number of vertices
    adjacent: HashMap<usize, Vec<(usize, usize)>>, // Adjacency list
}

impl Graph {
    fn new(ng: usize, nkeys: usize) -> Self {
        Self {
            n: nkeys,
            ng,
            adjacent: HashMap::new(),
        }
    }

    fn connect(&mut self, vertex1: usize, vertex2: usize, edge_value: usize) {
        self.adjacent.entry(vertex1).or_default().push((vertex2, edge_value));
        self.adjacent.entry(vertex2).or_default().push((vertex1, edge_value));
    }

    fn assign_vertex_values(&self) -> Option<Vec<usize>> {
        let mut vertex_values = vec![None; self.ng];
        let mut visited = vec![false; self.ng];

        if self.adjacent.len() <= self.n {
            // println!("only {} nodes", self.adjacent.len());
            return None;
        }

        for root in 0..self.ng {
            if visited[root] {
                continue;
            }

            vertex_values[root] = Some(0);
            let mut stack = vec![(None, root)];

            while let Some((parent, vertex)) = stack.pop() {
                visited[vertex] = true;

                let mut skip = true; // important to detect if f1(a) = f2(b) & f1(b) = f2(a)
                if let Some(neighbors) = self.adjacent.get(&vertex) {
                    for &(neighbor, edge_value) in neighbors {
                        if skip && Some(neighbor) == parent {
                            skip = false;
                            continue;
                        }

                        if visited[neighbor] {
                            return None; // Graph is cyclic
                        }

                        stack.push((Some(vertex), neighbor));

                        if let Some(value) = vertex_values[vertex] {
                            vertex_values[neighbor] = Some((edge_value + self.ng - value) % self.ng);
                        }
                    }
                }
            }
        }
        Some(vertex_values.into_iter().map(|v| v.unwrap()).collect::<Vec<usize>>())
    }
}


struct BaseHash {
    ng: usize,
    salt: Vec<u8>,
}

impl BaseHash {

    fn new(ng: usize, max_size: usize) -> Self {
        let mut salt = Vec::with_capacity(max_size);
        for _ in 0..max_size {
            // salt.push(rand::thread_rng().gen_range(0..ng));
            let u: u8 = rand::thread_rng().gen();
            salt.push(u);
        }
        while salt.len() % CHUNK_SIZE != 0 {
            salt.push(0);
        }
        BaseHash { 
            ng,
            salt
         }
    }

    // fn hash(&self, key: &str) -> usize {
    //     let mut hash: usize = 0;
    //     for (i, &byte) in key.as_bytes().iter().enumerate() {
    //         hash = hash.wrapping_add(byte as usize * self.salt[i]);
    //     }
    //     hash % self.ng
    // }

    // #[inline(always)]
    // fn hash(&self, key: &[u8]) -> usize {
    //     key.iter().zip(self.salt.iter())
    //         .map(|(a, b)| (*a as usize).overflowing_mul(*b).0)
    //         .sum::<usize>()
    //         % self.ng
    // }

    // #[inline]
    // fn hash(&self, key: &[u8]) -> usize {
    //     key.iter().zip(self.salt.iter())
    //         .map(|(a, b)| (*a as usize).overflowing_mul(*b as usize).0)
    //         .sum::<usize>()
    //         & (self.ng - 1)
    //         // % self.ng
    // }

    #[inline]
    fn hash(&self, key: &[u8]) -> usize {
        let key_bytes = key.chunks(CHUNK_SIZE);
        let salt = self.salt.chunks_exact(CHUNK_SIZE);

        let init: Simd<u8, CHUNK_SIZE> = Simd::from_array([1, 2, 3, 4, 5, 6, 7, 0]);
        let hash = key_bytes.zip(salt).fold(init, |acc, (a, b)| {
            let a: Simd<u8, CHUNK_SIZE> = Simd::load_or_default(a);
            let b = Simd::from_slice(b);
            // acc ^ (a * b) // overflow is the default behavior with SIMD
            acc ^ (a * b) // overflow is the default behavior with SIMD
            // acc.wrapping_add(result.reduce_sum())
        });
        let h: usize = unsafe { std::mem::transmute(hash) };
        h % self.ng
        // h & (self.ng - 1)
    }

}

#[pyclass(unsendable, name="Hash")]
struct Hash {
    ng: usize,
    f1: BaseHash,
    f2: BaseHash,
    indices: Vec<usize>
}


#[pymethods]
impl Hash {

    fn dump(&self) -> (usize, Vec<u8>,Vec<u8>, Vec<usize>) {
        (self.ng, self.f1.salt.clone(), self.f2.salt.clone(), self.indices.clone())
    }


    #[inline(always)]
    fn hash(&self, key: &str) -> usize {
        let h1 = self.f1.hash(key.as_bytes());
        let h2 = self.f2.hash(key.as_bytes());
        let combined_index = self.indices[h1].wrapping_add(self.indices[h2]);
        // combined_index & (self.ng - 1)
        combined_index % self.ng
    }


    #[inline]
    // #[call]
    #[pyo3(signature = (key))]
    fn __call__(&self, key: Bound<'_, PyString>) -> usize {
        let data = unsafe { key.data().unwrap() };
        let h1 = self.f1.hash(data.as_bytes());
        let h2 = self.f2.hash(data.as_bytes());
        let combined_index = self.indices[h1].wrapping_add(self.indices[h2]);
        combined_index % self.ng
        // assert!((combined_index & (self.ng - 1)) == combined_index % self.ng);
        // combined_index & (self.ng - 1)
    }

}


fn log2_ceil(x: usize) -> u32 {
    if x <= 1 {
        return 0; // log2_ceil(0 or 1) is 0
    }
    let bits = usize::BITS; // Number of bits in usize (platform-dependent)
    let floor_log2 = bits - x.leading_zeros() - 1;
    floor_log2 + 1
}


#[pyfunction]
fn generate_hasher(keys: Bound<'_, PyList>, values: Vec<usize>) ->  Result<Hash, PyErr> {
    let py_list = keys.downcast::<pyo3::types::PyList>()?;
    let mut keys = Vec::with_capacity(py_list.len());

    // copy the raw bytes of the strings
    for item in py_list {
        let py_str = item.downcast::<pyo3::types::PyString>()?;
        let data = unsafe { py_str.data().unwrap() };
        let copy = data.as_bytes().to_vec();
        keys.push(copy);
    }

    // read the algorithm description here
    // http://ilan.schnell-web.net/prog/perfect-hash/algo.html
    let mut trials = 0;
    let n = keys.len();
    let mut ng = keys.len() + 1;
    // let mut ng =  2usize.pow(log2_ceil(n));
    // if ng - n < n/2 {
    //     ng = 2 * ng;
    // }
    println!("Building a graph with {ng} vertices for a map of {n} elements");

    let max_size = keys.iter().map(|x| x.len()).fold(usize::MIN, |acc, a| a.max(acc));

    let (f1, f2, vertex_values) = loop {
        trials += 1;
        if trials % 8 == 0 { ng = ng + ng/8 + 1; }
        // if trials > 128 {
        //     ng *= 2;
        //     trials = 0;
        //     println!("doubling ng");
        // } else {
        //     trials += 1;
        // }

        python_interupt!(trials, 8);

        let mut graph = Graph::new(ng, n);
        let f1 = BaseHash::new(ng, max_size);
        let f2 = BaseHash::new(ng, max_size);

        for (key, hashval) in keys.iter().zip(values.iter()) {
            let h1 = f1.hash(key);
            let h2 = f2.hash(key);
            // println!("{h1}, {h2}");
            graph.connect(h1, h2, *hashval);
        }
        // panic!("");

        if let Some(vertex_values) = graph.assign_vertex_values() {
            println!("found a solution with ng : {ng}");
            break (f1, f2, vertex_values);
        }
    };

    Ok(Hash {
        ng,
        f1: f1,
        f2: f2,
        indices: vertex_values,
    })
}


#[pyfunction]
fn from_args(ng: usize, f1: Vec<u8>, f2: Vec<u8>, indices: Vec<usize>) -> Hash {

    let f1 = BaseHash { ng: ng, salt: f1 };
    let f2 = BaseHash { ng: ng, salt: f2 };

    Hash {
        ng,
        f1: f1,
        f2: f2,
        indices: indices,
    }
}


#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate_hasher, m)?)?;
    m.add_function(wrap_pyfunction!(from_args, m)?)?;
    Ok(())
}
