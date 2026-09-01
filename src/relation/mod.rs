pub(crate) mod bitset;
pub(crate) mod handle;
pub(crate) mod registry;

use pyo3::prelude::*;

use registry::NodeIndex;

#[pymodule]
pub mod relation {
    #[pymodule_export]
    use super::{
        handle::{RelationBitsetIterator, RelationHandle, RelationVecIterator},
        registry::{CutType, FlagHandle, RelationRegistry},
    };
}
