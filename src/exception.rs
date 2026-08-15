use pyo3::prelude::*;
use pyo3::{create_exception, exceptions::PyRuntimeError};

create_exception!(janim_backend, JAnimBackendException, PyRuntimeError);
create_exception!(janim_backend, LifetimeError, JAnimBackendException);
create_exception!(janim_backend, RelationCycleError, JAnimBackendException);
create_exception!(janim_backend, ReindexError, JAnimBackendException);

#[pymodule]
pub mod exception {
    #[pymodule_export]
    use super::{JAnimBackendException, RelationCycleError};
}
