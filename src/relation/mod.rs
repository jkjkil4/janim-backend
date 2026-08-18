pub(crate) mod bitset;
pub(crate) mod cmpt_bind;
pub(crate) mod handle;
pub(crate) mod registry;

use pyo3::prelude::*;

use registry::NodeIndex;

#[pymodule]
pub mod relation {
    #[pymodule_export]
    use super::{
        cmpt_bind::{BinderHandle, PyBindInfo},
        handle::{RelationBitsetIterator, RelationHandle, RelationVecIterator},
        registry::{FlagHandle, RelationRegistry},
    };

    #[pymodule_export]
    pub const FLAG_HANDLE_NAME: &str = "__flag_handle";
}
