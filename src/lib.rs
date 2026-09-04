#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]
#![deny(warnings)]

use pyo3::prelude::*;

pub mod ast;
pub mod parser;
pub mod prelude;
pub mod py_dict;

#[pymodule(name = "foam")]
pub mod foam {
    use super::*;

    #[pymodule_export]
    use py_dict::PyFoamDict as FoamDict;
}
