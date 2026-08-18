use pyo3::prelude::*;

use super::OffsetBitSet;
use super::{NodeIndex, RelationRegistry};

/// Parameter used when calling `RelationRegistry.node_set_flag`
///
/// The flag consists two parts,
/// `flag_0` (i.e. Component key) is provided externally,
/// `flag_1` (e.i. Component method name) is provided along with this `FlagHandle`.
#[pyclass(module = "janim_backend.relation", frozen, skip_from_py_object)]
pub struct FlagHandle {
    flag_1: usize,
    recurse_up: bool,
    recurse_down: bool,
}

impl FlagHandle {
    pub(super) fn new(flag_1: usize, recurse_up: bool, recurse_down: bool) -> Self {
        Self {
            flag_1,
            recurse_up,
            recurse_down,
        }
    }
}

impl RelationRegistry {
    /// Check for whether the flag of `index` is set
    pub fn node_get_computed_for(
        &self,
        index: NodeIndex,
        flag_0: usize,
        flag_handle: Bound<'_, FlagHandle>,
    ) -> bool {
        let flag_handle = flag_handle.borrow();
        match self
            .computed_flags
            .borrow_mut()
            .get(&(flag_0, flag_handle.flag_1))
        {
            Some(set) => set.contains(index),
            None => false,
        }
    }

    /// Reset the computed state to `true`, without considering the recursion
    pub fn node_mark_computed_for(
        &self,
        index: NodeIndex,
        flag_0: usize,
        flag_handle: Bound<'_, FlagHandle>,
    ) {
        let flag_handle = flag_handle.borrow();

        let mut flags = self.computed_flags.borrow_mut();
        let Some(set) = flags.get_mut(&(flag_0, flag_handle.flag_1)) else {
            return;
        };
        set.insert(index);
    }

    /// Set the computed state to `false`, considering the recursion
    pub fn node_reset_computed_for(
        &self,
        index: NodeIndex,
        flag_0: usize,
        flag_handle: PyRef<FlagHandle>,
    ) -> PyResult<()> {
        let mut flags = self.computed_flags.borrow_mut();
        let flag_set = flags.entry((flag_0, flag_handle.flag_1)).or_default();

        if !flag_handle.recurse_up && !flag_handle.recurse_down {
            flag_set.take(index);
        } else {
            let mut rel_set = OffsetBitSet::new();
            rel_set.insert(index);

            if flag_handle.recurse_up {
                rel_set.union_with(self.ancestor_set(index)?.as_ref());
            }
            if flag_handle.recurse_down {
                rel_set.union_with(self.descendant_set(index)?.as_ref());
            }

            flag_set.difference_with(&rel_set);
        }

        Ok(())
    }
}
