use pyo3::prelude::*;

use rand::thread_rng;
use rand::Rng;
use std::collections::HashMap;

struct Graph {
    n: usize,
    vertex_values: Vec<Option<usize>>,
    adjacent: HashMap<usize, Vec<(usize, usize)>>
}

impl Graph {

    fn new(n: usize) -> Self {
        assert!(n > 1);
        Graph {
            n,
            vertex_values: vec![None; n-1],
            adjacent: HashMap::new()
        }
    }


    fn connect(&mut self, vertex1: usize, vertex2: usize, edge_value: usize) {
        self.adjacent.entry(vertex1).or_insert_with(Vec::new).push((vertex2, edge_value));
        self.adjacent.entry(vertex2).or_insert_with(Vec::new).push((vertex1, edge_value));
    }

    fn assign_vertex_values(&mut self) -> bool {
        self.vertex_values = vec![None; self.n];
        let mut visited = vec![false; self.n];

        for root in 0..self.n {
            if visited[root] {
                continue;
            }

            self.vertex_values[root] = Some(0);
            let mut tovisit = vec![(None, root)];

            while let Some((parent, vertex)) = tovisit.pop() {
                visited[vertex] = true;

                let mut skip = true;
                for &(neighbor, edge_value) in &self.adjacent[&vertex] {
                    if skip && Some(neighbor) == parent {
                        skip = false;
                        continue;
                    }

                    if visited[neighbor] {
                        return false;
                    }

                    tovisit.push((Some(vertex), neighbor));
                    self.vertex_values[neighbor] = Some((edge_value + self.n - self.vertex_values[vertex].unwrap()) % self.n);
                }
            }
        }

        for value in &self.vertex_values {
            if value.is_none() {
                return false;
            }
        }
        true
    }
}


struct BaseHash {
    modulo: usize,
    salt: Vec<u8>
}

impl BaseHash {

    fn new(modulo: usize, max_size: usize) -> Self {
        let mut rng = thread_rng();
        let mut salt = Vec::with_capacity(max_size);
        for _ in 0..max_size {
            let u: u8 = rng.gen();
            salt.push(u);
        }
        BaseHash { 
            modulo: modulo,
            salt: salt
         }
    }


    fn call(&self, key: &String) -> usize {
        let bytes = key.as_bytes();
        assert!(self.salt.len() >= bytes.len());

        self.salt.iter()
        .zip(bytes)
        .fold(0, |acc, (a, b)| acc + (a.overflowing_mul(*b).0 as usize)) % self.modulo
    }

}

#[pyclass]
struct Hash {
    modulo: usize,
    right_bytes: Vec<u8>,
    left_bytes: Vec<u8>,
    indices: Vec<usize>
}


#[pymethods]
impl Hash {

    fn call(&self, key: String) -> usize {
        let bytes = key.as_bytes();

        // vertices[f1.call(key)] + vertices[f2.call(key)]

        // not correct
        bytes.iter().zip(self.right_bytes.iter()).zip(self.left_bytes.iter())
        .map(|((k, a), b) | (a.overflowing_mul(*k).0, b.overflowing_mul(*k).0))
        .fold(0, |acc, (a, b)| acc + (a as usize + b as usize)) % self.modulo
    }
}


#[pyfunction]
fn generate_hasher(keys: Vec<String>) -> Hash {

    let nk = keys.len();
    let ng = nk + 1;

    let max_size = keys.iter().map(|x| x.as_bytes().len()).fold(usize::MAX, |acc, a| a.min(acc));

    let (vertices, f1, f2) = loop {
        let mut graph = Graph::new(ng);
        let f1 = BaseHash::new(ng, max_size);
        let f2 = BaseHash::new(ng, max_size);

        for (index, key) in keys.iter().enumerate() {
            graph.connect(f1.call(key), f2.call(key), index);
        }

        if graph.assign_vertex_values() {
            let vertices: Vec<usize> = graph.vertex_values.into_iter().map(|x| x.unwrap()).collect();
            break (vertices, f1, f2);
        }
    };

    Hash {
        modulo: ng,
        right_bytes: f1.salt,
        left_bytes: f2.salt,
        indices: vertices
    }
}


#[pyfunction]
fn hello_from_bin() -> String {
    "Hello from perfect-hash!".to_string()
}


/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hello_from_bin, m)?)?;
    m.add_function(wrap_pyfunction!(generate_hasher, m)?)?;
    Ok(())
}
