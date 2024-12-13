use pyo3::prelude::*;

use rand::thread_rng;
use rand::Rng;
use std::collections::HashMap;

use pyo3::ffi::PyErr_CheckSignals;



// #[macro_export]
// // #[cfg(not(test))]
// macro_rules! python_interupt {
//     ($n_iter: expr, $period: expr) => {
//         if $n_iter % $period == 0 {
//             unsafe {if PyErr_CheckSignals() == -1 {panic!("Keyboard interupt");}}
//         }
//     };
// }

// // #[macro_export]
// // #[cfg(test)]
// // macro_rules! python_interupt {
// //     ($n_iter: expr, $period: expr) => {};
// // }


// struct Graph {
//     ng: usize,
//     vertex_values: Vec<Option<usize>>,
//     adjacent: HashMap<usize, Vec<(usize, usize)>>
// }

// impl Graph {

//     fn new(n: usize) -> Self {
//         assert!(n > 1);
//         Graph {
//             ng: n,
//             vertex_values: vec![None; n],
//             adjacent: HashMap::new()
//         }
//     }


//     fn connect(&mut self, vertex1: usize, vertex2: usize, edge_value: usize) {
//         // println!("connecting {:?} and {:?}", vertex1, vertex2);
//         self.adjacent.entry(vertex1).or_insert_with(Vec::new).push((vertex2, edge_value));
//         self.adjacent.entry(vertex2).or_insert_with(Vec::new).push((vertex1, edge_value));
//     }


//     fn assign_vertex_values(&mut self) -> bool {
//         self.vertex_values = vec![None; self.ng];
//         let mut visited = vec![false; self.ng];

//         for root in self.adjacent.keys() {
//             if visited[*root] {
//                 continue;
//             }

//             self.vertex_values[*root] = Some(42);
//             let mut tovisit = vec![(None, *root)];

//             while let Some((parent, vertex)) = tovisit.pop() {
//                 visited[vertex] = true;

//                 let mut skip = true;
//                 // println!("searching for key {:?}", vertex);
//                 // println!("adjacent is long : {:?}", self.adjacent.len());
//                 for &(neighbor, edge_value) in &self.adjacent[&vertex] {
//                     if skip && Some(neighbor) == parent {
//                         skip = false;
//                         continue;
//                     }

//                     if visited[neighbor] {
//                         return false;
//                     }

//                     tovisit.push((Some(vertex), neighbor));
//                     // +self.ng to avoid negative values with usize
//                     self.vertex_values[neighbor] = Some((edge_value + self.ng - self.vertex_values[vertex].unwrap()) % self.ng);
//                 }
//             }
//         }

//         // for value in &self.vertex_values {
//         //     if value.is_none() {
//         //         return false;
//         //     }
//         // }
//         true
//     }
// }


// struct BaseHash {
//     ng: usize,
//     salt: Vec<u8>
// }

// impl BaseHash {

//     fn new(modulo: usize, max_size: usize) -> Self {
//         let mut rng = thread_rng();
//         let mut salt = Vec::with_capacity(max_size);
//         for _ in 0..max_size {
//             let u: u8 = rng.gen();
//             salt.push(u);
//         }
//         BaseHash { 
//             ng: modulo,
//             salt
//          }
//     }


//     fn call(&self, key: &String) -> usize {
//         let bytes = key.as_bytes();
//         assert!(self.salt.len() >= bytes.len());

//         self.salt.iter()
//         .zip(bytes)
//         .fold(0, |acc, (a, b)| acc + (a.overflowing_mul(*b).0 as usize)) % self.ng
//     }

// }

// #[pyclass]
// struct Hash {
//     ng: usize,
//     right_bytes: Vec<u8>,
//     left_bytes: Vec<u8>,
//     indices: Vec<usize>
// }


// #[pymethods]
// impl Hash {

//     fn call(&self, key: String) -> usize {
//         let bytes = key.as_bytes();

//         // vertices[f1.call(key)] + vertices[f2.call(key)] % ng
//         let (a, b) = bytes.iter().zip(self.right_bytes.iter()).zip(self.left_bytes.iter())
//         .map(|((k, a), b) | (a.overflowing_mul(*k).0, b.overflowing_mul(*k).0))
//         .fold((0, 0), |(x, y), (a, b)| ((x + a as usize), (y + b as usize)));

//         (self.indices[a % self.ng] + self.indices[b % self.ng]) % self.ng
//     }
// }


// #[pyfunction]
// fn generate_hasher(keys: Vec<String>) -> Hash {
//     // read the algorithm description here
//     // http://ilan.schnell-web.net/prog/perfect-hash/algo.html

//     let mut ng: usize = keys.len() + 1;
//     let mut iter: usize = 0;
//     let mut max_iter: usize = 32;

//     let max_size = keys.iter().map(|x| x.as_bytes().len()).fold(usize::MIN, |acc, a| a.max(acc));


//     let (vertices, f1, f2) = loop {
//         iter += 1;
//         if iter > max_iter {
//             ng = ng + ng/4 + 1;
//             max_iter *= 2;
//         }

//         python_interupt!(iter, 8);

//         let mut graph = Graph::new(ng);
//         let f1 = BaseHash::new(ng, max_size);
//         let f2 = BaseHash::new(ng, max_size);

//         for (index, key) in keys.iter().enumerate() {
//             graph.connect(f1.call(key), f2.call(key), index);
//         }

//         if graph.assign_vertex_values() {
//             let vertices: Vec<usize> = graph.vertex_values.into_iter().map(|x| x.unwrap_or_default()).collect();
//             break (vertices, f1, f2);
//         }
//     };

//     println!("generated in {} iterations", iter);

//     Hash {
//         ng: ng,
//         right_bytes: f1.salt,
//         left_bytes: f2.salt,
//         indices: vertices
//     }
// }



#[derive(Debug)]
struct Graph {
    n: usize, // Number of vertices
    adjacent: HashMap<usize, Vec<(usize, usize)>>, // Adjacency list
}

impl Graph {
    fn new(n: usize) -> Self {
        Self {
            n,
            adjacent: HashMap::new(),
        }
    }

    fn connect(&mut self, vertex1: usize, vertex2: usize, edge_value: usize) {
        self.adjacent.entry(vertex1).or_default().push((vertex2, edge_value));
        self.adjacent.entry(vertex2).or_default().push((vertex1, edge_value));
    }

    fn assign_vertex_values(&self) -> Option<Vec<usize>> {
        let mut vertex_values = vec![None; self.n];
        let mut visited = vec![false; self.n];

        for root in 0..self.n {
            if visited[root] {
                continue;
            }

            vertex_values[root] = Some(0);
            let mut stack = vec![(None, root)];

            while let Some((parent, vertex)) = stack.pop() {
                visited[vertex] = true;

                if let Some(neighbors) = self.adjacent.get(&vertex) {
                    for &(neighbor, edge_value) in neighbors {
                        if Some(neighbor) == parent {
                            continue;
                        }

                        if visited[neighbor] {
                            return None; // Graph is cyclic
                        }

                        stack.push((Some(vertex), neighbor));

                        if let Some(value) = vertex_values[vertex] {
                            vertex_values[neighbor] = Some((edge_value + self.n - value) % self.n);
                        }
                    }
                }
            }
        }
        Some(vertex_values.into_iter().map(|v| v.unwrap()).collect::<Vec<usize>>())
    }
}

struct IntSaltHash {
    n: usize,
    salt: Vec<usize>,
}

impl IntSaltHash {
    fn new(n: usize) -> Self {
        Self { n, salt: Vec::new() }
    }

    fn hash(&mut self, key: &str) -> usize {
        while self.salt.len() < key.len() {
            self.salt.push(rand::thread_rng().gen_range(1..self.n));
        }

        key.chars()
            .enumerate()
            .map(|(i, c)| self.salt[i] * (c as usize))
            .sum::<usize>()
            % self.n
    }
}


#[pyclass]
struct Hash {
    ng: usize,
    f1: IntSaltHash,
    f2: IntSaltHash,
    indices: Vec<usize>
}


#[pymethods]
impl Hash {

    fn call(&mut self, key: String) -> usize {
        let h1 = self.f1.hash(&key);
        let h2 = self.f2.hash(&key);
        (self.indices[h1] + self.indices[h2]) % self.ng
    }
}


#[pyfunction]
fn generate_hasher(keys: Vec<String>) -> Hash {
    let mut trials = 0;
    let mut ng = keys.len() + 1;

    let (f1, f2, vertex_values) = loop {
        if trials % 5 == 0 && trials > 0 {
            ng = usize::max(ng + 1, (1.05 * ng as f64) as usize);
        }
        trials += 1;

        if ng > 100 * (keys.len() + 1) {
            panic!("Too many iterations");
        }

        let mut graph = Graph::new(ng);
        let mut f1 = IntSaltHash::new(ng);
        let mut f2 = IntSaltHash::new(ng);

        for (hashval, key) in keys.iter().enumerate() {
            let h1 = f1.hash(key);
            let h2 = f2.hash(key);
            graph.connect(h1, h2, hashval);
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
