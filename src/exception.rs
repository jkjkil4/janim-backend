use pyo3::prelude::*;
use pyo3::{create_exception, exceptions::PyRuntimeError};

create_exception!(janim_backend, JAnimBackendException, PyRuntimeError);

create_exception!(janim_backend, LifetimeError, JAnimBackendException);
create_exception!(janim_backend, BorrowMutError, JAnimBackendException);

create_exception!(janim_backend, RelationError, JAnimBackendException);
create_exception!(janim_backend, QuaternionError, JAnimBackendException);

#[pymodule]
pub mod exception {
    #[pymodule_export]
    use super::{
        BorrowMutError, JAnimBackendException, LifetimeError, QuaternionError, RelationError,
    };
}
