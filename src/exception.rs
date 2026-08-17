use pyo3::prelude::*;
use pyo3::{create_exception, exceptions::PyRuntimeError};

create_exception!(janim_backend, JAnimBackendException, PyRuntimeError);
create_exception!(janim_backend, LifetimeError, JAnimBackendException);
create_exception!(janim_backend, RelationError, JAnimBackendException);
create_exception!(janim_backend, ReindexError, JAnimBackendException);
create_exception!(janim_backend, BorrowMutError, JAnimBackendException);

#[pymodule]
pub mod exception {
    #[pymodule_export]
    use super::{JAnimBackendException, LifetimeError, ReindexError, RelationError};
}
