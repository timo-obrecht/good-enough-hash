use pyo3::prelude::*;

use rand::Rng;
use core::panic;
use std::collections::HashMap;

use pyo3::ffi::PyErr_CheckSignals;


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
    nkeys: usize,
    ng: usize, // Number of vertices
    adjacent: HashMap<usize, Vec<(usize, usize)>>, // Adjacency list
}

impl Graph {
    fn new(ng: usize, nkeys: usize) -> Self {
        Self {
            nkeys,
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

        if self.adjacent.len() <= self.nkeys {
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
    salt: Vec<usize>,
}

impl BaseHash {

    fn new(ng: usize, max_size: usize) -> Self {
        let mut salt = Vec::with_capacity(max_size);
        for _ in 0..max_size {
            salt.push(rand::thread_rng().gen_range(0..ng));
        }
        BaseHash { 
            ng,
            salt
         }
    }

    fn hash(&self, key: &str) -> usize {
        key.as_bytes()
            .iter().zip(self.salt.iter())
            .map(|(a, b)| (*a as usize).overflowing_mul(*b).0)
            .sum::<usize>()
            % self.ng
    }
}

#[pyclass]
struct Hash {
    ng: usize,
    f1: BaseHash,
    f2: BaseHash,
    indices: Vec<usize>
}


#[pymethods]
impl Hash {
    // fn call(&self, key: String) -> usize {
    //     let h1 = self.f1.hash(&key);
    //     let h2 = self.f2.hash(&key);
    //     (self.indices[h1] + self.indices[h2]) % self.ng
    // }

    // check if one is faster than the other
    // fn call(&self, key: String) -> usize {
    //     let bytes = key.as_bytes();
    //     let right_bytes = &self.f1.salt;
    //     let left_bytes = &self.f2.salt;

    //     let (a, b) = bytes.iter().zip(right_bytes.iter()).zip(left_bytes.iter())
    //     .map(|((k, a), b) | ((*k as usize) * a, (*k as usize)* b))
    //     .fold((0, 0), |(x, y), (a, b)| ((x + a), (y + b)));

    //     (self.indices[a % self.ng] + self.indices[b % self.ng]) % self.ng
    // }

}


#[pyfunction]
fn generate_hasher(keys: Vec<String>, values: Vec<usize>) -> Hash {
    // read the algorithm description here
    // http://ilan.schnell-web.net/prog/perfect-hash/algo.html
    let mut trials = 0;
    let mut ng = keys.len() + 1;
    let max_size = keys.iter().map(|x| x.as_bytes().len()).fold(usize::MIN, |acc, a| a.max(acc));

    let (f1, f2, vertex_values) = loop {
        trials += 1;
        if trials % 8 == 0 { ng = ng + ng/8 + 1; }
        python_interupt!(trials, 8);

        if ng > 100 * (keys.len() + 1) {
            panic!("Too many iterations");
        }

        let mut graph = Graph::new(ng, keys.len());
        let f1 = BaseHash::new(ng, max_size);
        let f2 = BaseHash::new(ng, max_size);

        for (key, hashval) in keys.iter().zip(values.iter()) {
            let h1 = f1.hash(key);
            let h2 = f2.hash(key);
            graph.connect(h1, h2, *hashval);
        }

        if let Some(vertex_values) = graph.assign_vertex_values() {
            break (f1, f2, vertex_values);
        }
    };

    Hash {
        ng,
        f1: f1,
        f2: f2,
        indices: vertex_values,
    }
}



#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate_hasher, m)?)?;
    Ok(())
}
