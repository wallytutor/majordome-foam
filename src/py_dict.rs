#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(warnings)]

use crate::ast::{FoamDict, FoamValue};
use crate::parser::parse_foam_dict;
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3::Py;
use std::fs;

/// Python class wrapper for OpenFOAM dictionary manipulation.
#[pyclass(name = "FoamDict")]
pub struct PyFoamDict {
    pub inner: FoamDict,
}

#[pymethods]
impl PyFoamDict {
    /// Create a new empty OpenFOAM dictionary.
    #[new]
    pub fn new() -> Self {
        Self {
            inner: FoamDict::new(),
        }
    }
}

impl Default for PyFoamDict {
    fn default() -> Self {
        Self::new()
    }
}

#[pymethods]
impl PyFoamDict {

    /// Parse OpenFOAM dictionary text into a `FoamDict` instance.
    #[staticmethod]
    pub fn parse(content: &str) -> PyResult<Self> {
        let dict = parse_foam_dict(content)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Ok(Self { inner: dict })
    }

    /// Load and parse OpenFOAM dictionary from a file path.
    #[staticmethod]
    pub fn from_file(path: &str) -> PyResult<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        Self::parse(&content)
    }

    /// Serialize dictionary into canonical OpenFOAM format string.
    pub fn to_foam(&self) -> String {
        self.inner.to_foam()
    }

    /// Write dictionary to a file.
    pub fn save(&self, path: &str) -> PyResult<()> {
        fs::write(path, self.to_foam())
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Get value by slash-separated key path (e.g. "solvers/p/tolerance").
    pub fn get(
        &self,
        py: Python<'_>,
        key_path: &str,
    ) -> PyResult<Option<Py<PyAny>>> {
        match self.inner.get_path(key_path) {
            Some(val) => {
                let py_obj = foam_value_to_py(py, &val)?;
                Ok(Some(py_obj))
            }
            None => Ok(None),
        }
    }

    /// Set value by slash-separated key path.
    pub fn set(
        &mut self,
        key_path: &str,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let foam_val = py_to_foam_value(value)?;
        self.inner.set_path(key_path, foam_val);

        Ok(())
    }

    /// Remove entry by slash-separated key path.
    pub fn delete(&mut self, key_path: &str) -> bool {
        self.inner.delete_path(key_path)
    }

    /// Check if key path exists in dictionary.
    pub fn contains(&self, key_path: &str) -> bool {
        self.inner.get_path(key_path).is_some()
    }

    /// Add `#include "filename"` directive to the dictionary.
    pub fn add_include(&mut self, filename: &str) {
        self.inner.add_include(filename);
    }

    /// Add `#includeEtc "filename"` directive to the dictionary.
    pub fn add_include_etc(&mut self, filename: &str) {
        self.inner.add_include_etc(filename);
    }

    /// Return list of top-level keys in dictionary.
    pub fn keys(&self) -> Vec<String> {
        self.inner.keys()
    }

    fn __getitem__(
        &self,
        py: Python<'_>,
        key_path: &str,
    ) -> PyResult<Py<PyAny>> {
        match self.get(py, key_path)? {
            Some(obj) => Ok(obj),
            None => {
                Err(PyKeyError::new_err(format!("Key not found: {}", key_path)))
            }
        }
    }

    fn __setitem__(
        &mut self,
        key_path: &str,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        self.set(key_path, value)
    }

    fn __contains__(&self, key_path: &str) -> bool {
        self.contains(key_path)
    }
}

fn foam_value_to_py(
    py: Python<'_>,
    val: &FoamValue,
) -> PyResult<Py<PyAny>> {
    match val {
        FoamValue::Scalar(v) => {
            Ok(v.into_pyobject(py)?.to_owned().into_any().unbind())
        }
        FoamValue::Int(v) => {
            Ok(v.into_pyobject(py)?.to_owned().into_any().unbind())
        }
        FoamValue::String(v) => {
            Ok(v.as_str().into_pyobject(py)?.to_owned().into_any().unbind())
        }
        FoamValue::Bool(v) => {
            Ok(v.into_pyobject(py)?.to_owned().into_any().unbind())
        }
        FoamValue::Vector(v) => {
            Ok(v.clone().into_pyobject(py)?.into_any().unbind())
        }
        FoamValue::DimensionSet(v) => {
            Ok(v.clone().into_pyobject(py)?.into_any().unbind())
        }

        FoamValue::List(items) => {
            let py_list = PyList::empty(py);

            for item in items {
                py_list.append(foam_value_to_py(py, item)?)?;
            }

            Ok(py_list.into_any().unbind())
        }

        FoamValue::Dict(d) => {
            let py_dict = PyFoamDict { inner: d.clone() };
            Ok(py_dict.into_pyobject(py)?.into_any().unbind())
        }

        FoamValue::MacroRef(m) => {
            Ok(m.as_str().into_pyobject(py)?.to_owned().into_any().unbind())
        }
        FoamValue::Raw(r) => {
            Ok(r.as_str().into_pyobject(py)?.to_owned().into_any().unbind())
        }
    }
}

fn py_to_foam_value(value: &Bound<'_, PyAny>) -> PyResult<FoamValue> {
    if let Ok(b) = value.extract::<bool>() {
        return Ok(FoamValue::Bool(b));
    }

    if let Ok(i) = value.extract::<i64>() {
        return Ok(FoamValue::Int(i));
    }

    if let Ok(f) = value.extract::<f64>() {
        return Ok(FoamValue::Scalar(f));
    }

    if let Ok(s) = value.extract::<String>() {
        return Ok(FoamValue::String(s));
    }

    if let Ok(list) = value.extract::<Vec<Bound<'_, PyAny>>>() {
        let mut vec = Vec::new();

        for item in &list {
            vec.push(py_to_foam_value(item)?);
        }

        return Ok(FoamValue::List(vec));
    }

    if let Ok(dict) = value.extract::<PyRef<'_, PyFoamDict>>() {
        return Ok(FoamValue::Dict(dict.inner.clone()));
    }

    Err(PyValueError::new_err(format!(
        "Unsupported type for FoamValue: {}",
        value
    )))
}
