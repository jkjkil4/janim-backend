use std::rc::Rc;

use pyo3::prelude::*;
use pyo3::types::PyWeakrefReference;

use crate::exception::RelationError;
use crate::utils::upgrade_ref;

use super::bitset::OffsetBitSet;
use super::cache::RecursiveCache;
use super::handle::RelationHandle;
use super::{NodeIndex, RelationRegistry};

impl RelationRegistry {
    /// Add child objects
    pub(super) fn add_children_to(
        &mut self,
        py: Python<'_>,
        root: &RelationHandle,
        new_children: Vec<Bound<'_, RelationHandle>>,
        prepend: bool,
    ) -> PyResult<()> {
        let (index, obj_ref) = root.index_and_ref(py)?;
        let self_children = root.children_ref(py);

        self.children_changed(py, index)?;
        for child in new_children {
            let (child_index, child_obj) = child.borrow().index_and_ref(py)?;

            let node = self.node_mut(index);
            if node.has_child(child_index) {
                continue;
            }

            let add_to_children = (child_index, child_obj.unbind());
            let add_to_parents = (index, obj_ref.clone());

            // Add to Rust index vec & python object list
            if prepend {
                node.children.insert(0, add_to_children.0);
                self_children.insert(0, add_to_children.1).unwrap();
            } else {
                node.children.push(add_to_children.0);
                self_children.append(add_to_children.1).unwrap();
            }

            // Add to Rust index vec & python object list
            let child_node = self.node_mut(child_index);
            child_node.parents.push(add_to_parents.0);
            child_node
                .unwrap_handle(py)
                .parents_ref(py)
                .append(add_to_parents.1)
                .unwrap();

            self.parents_changed(py, child_index)?;
        }
        Ok(())
    }

    // Insert child objects
    pub(super) fn insert_children_to(
        &mut self,
        py: Python<'_>,
        root: &RelationHandle,
        insert_index: usize,
        new_children: Vec<Bound<'_, RelationHandle>>,
    ) -> PyResult<()> {
        let (index, obj_ref) = root.index_and_ref(py)?;
        let self_children = root.children_ref(py);

        let insert_index = insert_index.min(self.node(index).children.len());

        self.children_changed(py, index)?;
        for (i, child) in new_children.into_iter().enumerate() {
            let (child_index, child_obj) = child.borrow().index_and_ref(py)?;

            let node = self.node_mut(index);
            if node.has_child(child_index) {
                continue;
            }

            let add_to_children = (child_index, child_obj.unbind());
            let add_to_parents = (index, obj_ref.clone());

            // Add to Rust index vec & python object list
            node.children.insert(insert_index + i, add_to_children.0);
            self_children
                .insert(insert_index + i, add_to_children.1)
                .unwrap();

            // Add to Rust index vec & python object list
            let child_node = self.node_mut(child_index);
            child_node.parents.push(add_to_parents.0);
            child_node
                .unwrap_handle(py)
                .parents_ref(py)
                .append(add_to_parents.1)
                .unwrap();

            self.parents_changed(py, child_index)?;
        }
        Ok(())
    }

    // Remove child objects
    pub(super) fn remove_children_from(
        &mut self,
        py: Python<'_>,
        root: &RelationHandle,
        removed_children: Vec<Bound<'_, RelationHandle>>,
    ) -> PyResult<()> {
        self.remove_by_index(
            py,
            root.index(),
            &removed_children
                .into_iter()
                .map(|handle| handle.borrow().index())
                .collect(),
        )
    }

    /// Clear parent objects
    pub(super) fn clear_parents_of(
        &mut self,
        py: Python<'_>,
        root: &RelationHandle,
    ) -> PyResult<()> {
        let index = root.index();

        for parent_index in self.node(index).parents.clone() {
            self.remove_by_index(py, parent_index, &vec![index])?;
        }
        Ok(())
    }

    /// Clear child objects
    pub(super) fn clear_children_of(
        &mut self,
        py: Python<'_>,
        root: &RelationHandle,
    ) -> PyResult<()> {
        let index = root.index();

        self.remove_by_index(py, index, &self.node(index).children.clone())
    }
}

impl RelationRegistry {
    #[inline]
    pub(super) fn node(&self, node_index: NodeIndex) -> &Node {
        &self.nodes[node_index - self.offset]
    }

    #[inline]
    pub(super) fn node_mut(&mut self, node_index: NodeIndex) -> &mut Node {
        &mut self.nodes[node_index - self.offset]
    }

    fn remove_by_index(
        &mut self,
        py: Python<'_>,
        index: NodeIndex,
        removed_children: &Vec<NodeIndex>,
    ) -> PyResult<()> {
        self.children_changed(py, index)?;
        let self_children = self.node(index).unwrap_handle(py).children_ref(py).clone();

        for &child_index in removed_children {
            let node = self.node_mut(index);
            if !node.has_child(child_index) {
                continue;
            }

            // Remove from Rust index vec & python object list
            let Some(i) = node.children.iter().position(|&x| x == child_index) else {
                continue;
            };
            node.children.remove(i);
            self_children.del_item(i).unwrap();

            // Remove from Rust index vec & python object list
            let child_parents = &mut self.node_mut(child_index).parents;
            let i = child_parents.iter().position(|&x| x == index).unwrap();
            child_parents.remove(i);
            self.node(child_index)
                .unwrap_handle(py)
                .parents_ref(py)
                .del_item(i)
                .unwrap();

            self.parents_changed(py, child_index)?;
        }
        Ok(())
    }

    fn parents_changed(&self, py: Python<'_>, index: NodeIndex) -> PyResult<()> {
        let node = self.node(index);
        node.reset_ancestors();
        let descendants = self.descendant_set(index)?;
        for child in descendants.iter() {
            self.node(child).reset_ancestors();
        }
        // Call the python callback
        node.resolve_self(py)?
            .unwrap()
            .getattr("_parents_changed")?
            .call0()?;
        Ok(())
    }

    fn children_changed(&self, py: Python<'_>, index: NodeIndex) -> PyResult<()> {
        let node = self.node(index);
        node.reset_descendants();
        let ancestors = self.ancestor_set(index)?;
        for child in ancestors.iter() {
            self.node(child).reset_descendants();
        }
        // Call the python callback
        node.resolve_self(py)?
            .unwrap()
            .getattr("_children_changed")?
            .call0()?;
        Ok(())
    }

    /// Get the ancestor bitset with cache,
    /// `Err` if the graph has cycle
    pub(super) fn ancestor_set(&self, node_index: NodeIndex) -> PyResult<Rc<OffsetBitSet>> {
        let node = self.node(node_index);
        self.cached_set(&node.ancestor_set, &node.parents, Self::ancestor_set)
    }

    /// Get the descendant bitset with cache,
    /// `Err` if the graph has cycle
    pub(super) fn descendant_set(&self, node_index: NodeIndex) -> PyResult<Rc<OffsetBitSet>> {
        let node = self.node(node_index);
        self.cached_set(&node.descendant_set, &node.children, Self::descendant_set)
    }

    /// `Err` if the graph has cycle
    fn cached_set(
        &self,
        cache: &RecursiveCache<OffsetBitSet>,
        neighbors: &Vec<usize>,
        recurse: impl Fn(&Self, NodeIndex) -> PyResult<Rc<OffsetBitSet>>,
    ) -> PyResult<Rc<OffsetBitSet>> {
        cache.get_or_compute(
            || {
                let mut set = OffsetBitSet::new();

                for &index in neighbors {
                    set.insert(index);

                    let subset = recurse(self, index)?;
                    set.union_with(subset.as_ref());
                }

                Ok(set)
            },
            new_cycle_err,
        )
    }

    /// Get the ancestor vec with cache,
    /// `Err` if the graph has cycle
    pub(super) fn ancestor_dfs(&self, node_index: NodeIndex) -> PyResult<Rc<Vec<NodeIndex>>> {
        let node = self.node(node_index);
        self.cached_dfs(&node.ancestor_dfs, &node.parents, Self::ancestor_dfs)
    }

    /// Get the descendant vec with cache,
    /// `Err` if the graph has cycle
    pub(super) fn descendant_dfs(&self, node_index: NodeIndex) -> PyResult<Rc<Vec<NodeIndex>>> {
        let node = self.node(node_index);
        self.cached_dfs(&node.descendant_dfs, &node.children, Self::descendant_dfs)
    }

    /// `Err` if the graph has cycle
    fn cached_dfs(
        &self,
        cache: &RecursiveCache<Vec<NodeIndex>>,
        neighbors: &Vec<NodeIndex>,
        recurse: impl Fn(&Self, NodeIndex) -> PyResult<Rc<Vec<NodeIndex>>>,
    ) -> PyResult<Rc<Vec<NodeIndex>>> {
        cache.get_or_compute(
            || {
                let mut res = Vec::new();

                for &index in neighbors {
                    if !res.contains(&index) {
                        res.push(index);
                    }
                    let filtered_recurse: Vec<NodeIndex> = recurse(self, index)?
                        .iter()
                        .filter(|x| !res.contains(x))
                        .copied()
                        .collect();
                    res.extend(filtered_recurse);
                }

                Ok(res)
            },
            new_cycle_err,
        )
    }
}

fn new_cycle_err() -> PyErr {
    RelationError::new_err(t!(
        "The relation graph contains a cycle. Consider checking the inclusion relations between groups."
    ))
}

pub(super) enum ResolveResult<'py> {
    Resolved(Bound<'py, PyAny>),
    Expired,
}

impl<'py> ResolveResult<'py> {
    fn unwrap(self) -> Bound<'py, PyAny> {
        match self {
            ResolveResult::Resolved(obj) => obj,
            ResolveResult::Expired => panic!(),
        }
    }
}

pub(super) struct Node {
    /// Used for checking whether a node is alive
    handle_wref: Py<PyWeakrefReference>,

    /// `parents` does not hold the python reference
    /// we allocate object list externally in the handle
    parents: Vec<NodeIndex>,
    /// `children` does not hold the python reference
    /// we allocate object list externally in the handle
    children: Vec<NodeIndex>,

    ancestor_set: RecursiveCache<OffsetBitSet>,
    descendant_set: RecursiveCache<OffsetBitSet>,

    ancestor_dfs: RecursiveCache<Vec<NodeIndex>>,
    descendant_dfs: RecursiveCache<Vec<NodeIndex>>,
}

impl Node {
    pub(super) fn new(handle: &Bound<'_, RelationHandle>) -> PyResult<Self> {
        Ok(Self {
            handle_wref: PyWeakrefReference::new(handle)?.unbind(),
            parents: Vec::new(),
            children: Vec::new(),
            ancestor_set: RecursiveCache::new(),
            descendant_set: RecursiveCache::new(),
            ancestor_dfs: RecursiveCache::new(),
            descendant_dfs: RecursiveCache::new(),
        })
    }

    pub(super) fn alive(&self, py: Python<'_>) -> bool {
        upgrade_ref(py, &self.handle_wref).is_some()
    }

    #[inline]
    fn has_child(&self, check: NodeIndex) -> bool {
        self.children.contains(&check)
    }

    fn unwrap_handle<'py>(&self, py: Python<'py>) -> PyRef<'py, RelationHandle> {
        upgrade_ref(py, &self.handle_wref)
            .unwrap()
            .extract()
            .unwrap()
    }

    fn reset_ancestors(&self) {
        self.ancestor_set.reset();
        self.ancestor_dfs.reset();
    }

    fn reset_descendants(&self) {
        self.descendant_set.reset();
        self.descendant_dfs.reset();
    }

    pub(super) fn resolve_self<'py>(&self, py: Python<'py>) -> PyResult<ResolveResult<'py>> {
        match upgrade_ref(py, &self.handle_wref) {
            Some(py_handle) => {
                let handle: PyRef<RelationHandle> = py_handle.extract().unwrap();
                Ok(ResolveResult::Resolved(handle.obj_ref(py)?))
            }
            None => Ok(ResolveResult::Expired),
        }
    }
}
