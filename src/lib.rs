use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::ffi::PyErr_CheckSignals;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyString};
use std::collections::HashMap;
use std::ptr;

#[macro_export]
#[cfg(not(test))]
macro_rules! python_interupt {
    ($n_iter: expr, $period: expr) => {
        if $n_iter % $period == 0 {
            unsafe {
                if PyErr_CheckSignals() == -1 {
                    panic!("Keyboard interupt");
                }
            }
        }
    };
}

#[macro_export]
#[cfg(test)]
macro_rules! python_interupt {
    ($n_iter: expr, $period: expr) => {};
}

#[inline(always)]
fn mix64(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

#[inline(always)]
fn hash_summary(key: &[u8], seed1: u64, seed2: u64, ng: usize) -> (usize, usize, u8) {
    let mut h1 = seed1 ^ ((key.len() as u64).wrapping_mul(0x9e3779b185ebca87));
    let mut h2 = seed2 ^ ((key.len() as u64).wrapping_mul(0xc2b2ae3d27d4eb4f));

    let mut index = 0usize;
    while index + 8 <= key.len() {
        let word = unsafe { ptr::read_unaligned(key.as_ptr().add(index) as *const u64) };
        h1 = mix64(h1 ^ word);
        h2 = mix64(h2.wrapping_add(word.rotate_left(17)));
        index += 8;
    }

    if index < key.len() {
        let mut word = 0u64;
        for (shift, byte) in key[index..].iter().enumerate() {
            word |= (*byte as u64) << (shift * 8);
        }
        h1 = mix64(h1 ^ word ^ 0x517cc1b727220a95);
        h2 = mix64(h2.wrapping_add(word ^ 0x6c8e9cf570932bd5));
    }

    let mixed1 = mix64(h1);
    let mixed2 = mix64(h2 ^ h1.rotate_left(32));
    let tag = (mix64(mixed1 ^ mixed2 ^ ((key.len() as u64) << 8)) & 0xff) as u8;
    let mask = ng - 1;
    let first = (mixed1 as usize) & mask;
    let second = (mixed2 as usize) & mask;
    (first, second, tag)
}

struct Graph {
    n: usize,
    ng: usize,
    adjacent: HashMap<usize, Vec<(usize, usize)>>,
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
        self.adjacent
            .entry(vertex1)
            .or_default()
            .push((vertex2, edge_value));
        self.adjacent
            .entry(vertex2)
            .or_default()
            .push((vertex1, edge_value));
    }

    fn assign_vertex_values(&self) -> Option<Vec<usize>> {
        let mut vertex_values = vec![None; self.ng];
        let mut visited = vec![false; self.ng];
        let mask = self.ng - 1;

        if self.adjacent.len() <= self.n {
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

                let mut skip = true;
                if let Some(neighbors) = self.adjacent.get(&vertex) {
                    for &(neighbor, edge_value) in neighbors {
                        if skip && Some(neighbor) == parent {
                            skip = false;
                            continue;
                        }

                        if visited[neighbor] {
                            return None;
                        }

                        stack.push((Some(vertex), neighbor));

                        if let Some(value) = vertex_values[vertex] {
                            vertex_values[neighbor] = Some(
                                edge_value.wrapping_add(self.ng).wrapping_sub(value) & mask,
                            );
                        }
                    }
                }
            }
        }

        Some(vertex_values.into_iter().map(|value| value.unwrap()).collect())
    }
}

#[pyclass(unsendable, name = "Hash")]
struct Hash {
    ng: usize,
    seed1: u64,
    seed2: u64,
    indices: Vec<usize>,
    values: Vec<usize>,
    key_tags: Vec<u8>,
    key_offsets: Vec<u32>,
    key_data: Vec<u8>,
}

#[pymethods]
impl Hash {
    fn dump(&self) -> (usize, u64, u64, Vec<usize>, Vec<usize>, Vec<u8>, Vec<u32>, Vec<u8>) {
        (
            self.ng,
            self.seed1,
            self.seed2,
            self.indices.clone(),
            self.values.clone(),
            self.key_tags.clone(),
            self.key_offsets.clone(),
            self.key_data.clone(),
        )
    }

    #[inline(always)]
    fn index_and_tag_for(&self, key: &[u8]) -> (usize, u8) {
        let (h1, h2, tag) = hash_summary(key, self.seed1, self.seed2, self.ng);
        (
            self.indices[h1].wrapping_add(self.indices[h2]) & (self.ng - 1),
            tag,
        )
    }

    #[inline(always)]
    fn contains_at_index(&self, index: usize, tag: u8, key: &[u8]) -> bool {
        if index >= self.values.len() {
            return false;
        }
        if self.key_tags[index] != tag {
            return false;
        }

        let start = self.key_offsets[index] as usize;
        let end = self.key_offsets[index + 1] as usize;
        &self.key_data[start..end] == key
    }

    #[inline]
    #[pyo3(signature = (key))]
    fn __call__(&self, key: Bound<'_, PyString>) -> PyResult<usize> {
        let data = unsafe { key.data().unwrap() };
        let bytes = data.as_bytes();
        let (index, tag) = self.index_and_tag_for(bytes);

        if self.contains_at_index(index, tag, bytes) {
            return Ok(self.values[index]);
        }

        Err(PyKeyError::new_err(key.extract::<String>()?))
    }
}

#[pyfunction]
fn generate_hasher(keys: Bound<'_, PyList>, values: Vec<usize>) -> Result<Hash, PyErr> {
    let py_list = keys.cast::<PyList>()?;
    let mut keys = Vec::with_capacity(py_list.len());

    for item in py_list {
        let py_str = item.cast::<PyString>()?;
        let data = unsafe { py_str.data().unwrap() };
        keys.push(data.as_bytes().to_vec());
    }

    if values.len() != keys.len() {
        return Err(PyValueError::new_err(
            "values must have the same length as keys",
        ));
    }

    if keys.is_empty() {
        return Ok(Hash {
            ng: 1,
            seed1: rand::random(),
            seed2: rand::random(),
            indices: vec![0],
            values,
            key_tags: Vec::new(),
            key_offsets: vec![0],
            key_data: Vec::new(),
        });
    }

    let total_key_bytes = keys.iter().map(|key| key.len()).sum::<usize>();
    if total_key_bytes > u32::MAX as usize {
        return Err(PyValueError::new_err(
            "total serialized key size exceeds supported limit",
        ));
    }

    let mut key_offsets = Vec::with_capacity(keys.len() + 1);
    let mut key_data = Vec::with_capacity(total_key_bytes);
    key_offsets.push(0);
    for key in &keys {
        key_data.extend_from_slice(key);
        key_offsets.push(key_data.len() as u32);
    }

    let mut trials = 0usize;
    let n = keys.len();
    let mut ng = (n + 1).next_power_of_two();

    let (seed1, seed2, vertex_values, key_tags) = loop {
        trials += 1;
        if trials % 8 == 0 {
            ng *= 2;
        }

        python_interupt!(trials, 8);

        let seed1: u64 = rand::random();
        let seed2: u64 = rand::random();
        let mut graph = Graph::new(ng, n);

        for (key_index, key) in keys.iter().enumerate() {
            let (h1, h2, _) = hash_summary(key, seed1, seed2, ng);
            graph.connect(h1, h2, key_index);
        }

        if let Some(vertex_values) = graph.assign_vertex_values() {
            let mut key_tags = vec![0u8; keys.len()];
            for (key_index, key) in keys.iter().enumerate() {
                let (_, _, tag) = hash_summary(key, seed1, seed2, ng);
                key_tags[key_index] = tag;
            }
            break (seed1, seed2, vertex_values, key_tags);
        }
    };

    Ok(Hash {
        ng,
        seed1,
        seed2,
        indices: vertex_values,
        values,
        key_tags,
        key_offsets,
        key_data,
    })
}

#[pyfunction]
fn from_args(
    ng: usize,
    seed1: u64,
    seed2: u64,
    indices: Vec<usize>,
    values: Vec<usize>,
    key_tags: Vec<u8>,
    key_offsets: Vec<u32>,
    key_data: Vec<u8>,
) -> Hash {
    Hash {
        ng,
        seed1,
        seed2,
        indices,
        values,
        key_tags,
        key_offsets,
        key_data,
    }
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate_hasher, m)?)?;
    m.add_function(wrap_pyfunction!(from_args, m)?)?;
    Ok(())
}
