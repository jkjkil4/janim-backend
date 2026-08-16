use pyo3::PyResult;

use super::bitset::OffsetBitSet;
use super::{NodeIndex, RelationRegistry};

impl RelationRegistry {
    /// Check for whether the flag of `index` is set
    pub(super) fn node_has_flag(&mut self, index: NodeIndex, flag: String) -> bool {
        self.flags.entry(flag).or_default().contains(index)
    }

    /// Set the flag state of `index`
    pub(super) fn node_set_flag(
        &mut self,
        index: NodeIndex,
        flag: String,
        state: bool,
        recurse_up: bool,
        recurse_down: bool,
    ) -> PyResult<()> {
        if !recurse_up && !recurse_down {
            let flag_set = self.flags.entry(flag).or_default();

            if state {
                flag_set.insert(index);
            } else {
                flag_set.take(index);
            }
        } else {
            let mut rel_set = OffsetBitSet::new();
            rel_set.insert(index);

            if recurse_up {
                rel_set.union_with(self.ancestor_set(index)?.as_ref());
            }
            if recurse_down {
                rel_set.union_with(self.descendant_set(index)?.as_ref());
            }

            let flag_set = self.flags.entry(flag).or_default();

            if state {
                flag_set.union_with(&rel_set);
            } else {
                flag_set.difference_with(&rel_set);
            }
        }

        Ok(())
    }
}
