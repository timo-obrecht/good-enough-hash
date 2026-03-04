mod mphf;

use mphf::{ActiveMphf, Mphf};
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::ffi::PyErr_CheckSignals;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyString};

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

#[pyclass(unsendable, name = "Hash")]
struct Hash {
    mphf: ActiveMphf,
    values: Vec<(Py<PyAny>, u64)>,
}

#[pymethods]
impl Hash {
    fn dump(&self, py: Python<'_>) -> PyResult<DumpState> {
        let (tag_seed, bucket_seed, bucket_count, table_len, pilots) = self.mphf.dump();
        let mut values = Vec::with_capacity(self.values.len());
        let mut tags = Vec::with_capacity(self.values.len());

        for (value, tag) in &self.values {
            values.push(value.clone_ref(py));
            tags.push(*tag);
        }

        Ok((
            tag_seed,
            bucket_seed,
            bucket_count,
            table_len,
            pilots,
            values,
            tags,
        ))
    }

    #[inline]
    #[pyo3(signature = (key))]
    fn __call__(&self, py: Python<'_>, key: Bound<'_, PyString>) -> PyResult<Py<PyAny>> {
        let data = unsafe { key.data().unwrap() };
        let bytes = data.as_bytes();
        let tag = self.mphf.tag(bytes);
        let index = self.mphf.index_from_tag(tag);

        if let Some((value, stored_tag)) = self.values.get(index) {
            if *stored_tag == tag {
                return Ok(value.clone_ref(py));
            }
        }

        Err(PyKeyError::new_err(key.extract::<String>()?))
    }
}

type DumpState = (
    u64,
    u64,
    usize,
    usize,
    Vec<u64>,
    Vec<Py<PyAny>>,
    Vec<u64>,
);

#[pyfunction]
fn generate_hasher(
    py: Python<'_>,
    keys: Bound<'_, PyList>,
    values: Bound<'_, PyList>,
) -> Result<Hash, PyErr> {
    let mut encoded_keys = Vec::with_capacity(keys.len());
    for item in keys.iter() {
        let py_str = item.cast::<PyString>()?;
        let data = unsafe { py_str.data().unwrap() };
        encoded_keys.push(data.as_bytes().to_vec());
    }

    let mut py_values = Vec::with_capacity(values.len());
    for value in values.iter() {
        py_values.push(value.unbind());
    }

    if py_values.len() != encoded_keys.len() {
        return Err(PyValueError::new_err(
            "values must have the same length as keys",
        ));
    }

    python_interupt!(encoded_keys.len().max(1), 1);
    let build = ActiveMphf::build(&encoded_keys)?;

    let mut slots: Vec<Option<(Py<PyAny>, u64)>> =
        (0..build.permutation.len()).map(|_| None).collect();

    for (slot, original_index) in build.permutation.into_iter().enumerate() {
        slots[slot] = Some((py_values[original_index].clone_ref(py), build.tags[original_index]));
    }

    Ok(Hash {
        mphf: build.mphf,
        values: slots
            .into_iter()
            .map(|slot| slot.expect("all MPHF slots should be assigned"))
            .collect(),
    })
}

#[pyfunction]
fn from_args(
    tag_seed: u64,
    bucket_seed: u64,
    bucket_count: usize,
    table_len: usize,
    pilots: Vec<u64>,
    values: Vec<Py<PyAny>>,
    tags: Vec<u64>,
) -> PyResult<Hash> {
    if values.len() != tags.len() {
        return Err(PyValueError::new_err(
            "values and tags must have the same length",
        ));
    }
    if values.len() != table_len {
        return Err(PyValueError::new_err(
            "values and tags must have the same length as table_len",
        ));
    }

    let mphf = ActiveMphf::from_dump(tag_seed, bucket_seed, bucket_count, table_len, pilots)?;
    let values = values.into_iter().zip(tags).collect();

    Ok(Hash { mphf, values })
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate_hasher, m)?)?;
    m.add_function(wrap_pyfunction!(from_args, m)?)?;
    Ok(())
}
