use std::rc::Rc;

use pyo3::prelude::*;
use pyo3::types::PyWeakrefReference;

use crate::exception::{ReindexError, RelationError};
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
        children: Vec<Bound<'_, RelationHandle>>,
        prepend: bool,
    ) -> PyResult<()> {
        let (index, wref) = root.index_and_wref();

        self.children_changed(index)?;
        for child in children {
            let (child_index, child_obj) = child.borrow().index_and_ref(py)?;

            let node = self.node_mut(index);
            if node.has_child(child_index) {
                continue;
            }

            let add_to_children = (child_index, child_obj.unbind());
            let add_to_parents = (index, wref.clone_ref(py));
            if prepend {
                node.children.insert(0, add_to_children);
                self.node_mut(child_index).parents.push(add_to_parents);
            } else {
                node.children.push(add_to_children);
                self.node_mut(child_index).parents.push(add_to_parents);
            }
            self.parents_changed(child_index)?;
        }
        Ok(())
    }

    // Insert child objects
    pub(super) fn insert_children_to(
        &mut self,
        py: Python<'_>,
        root: &RelationHandle,
        insert_index: usize,
        children: Vec<Bound<'_, RelationHandle>>,
    ) -> PyResult<()> {
        let (index, wref) = root.index_and_wref();

        let insert_index = insert_index.min(self.node(index).children.len());

        self.children_changed(index)?;
        for (i, child) in children.into_iter().enumerate() {
            let (child_index, child_obj) = child.borrow().index_and_ref(py)?;

            let node = self.node_mut(index);
            if node.has_child(child_index) {
                continue;
            }

            let add_to_children = (child_index, child_obj.unbind());
            let add_to_parents = (index, wref.clone_ref(py));

            node.children.insert(insert_index + i, add_to_children);
            self.node_mut(child_index).parents.push(add_to_parents);

            self.parents_changed(child_index)?;
        }
        Ok(())
    }

    // Remove child objects
    pub(super) fn remove_children_from(
        &mut self,
        root: &RelationHandle,
        children: Vec<Bound<'_, RelationHandle>>,
    ) -> PyResult<()> {
        self.remove_by_index(
            root.index(),
            &children
                .into_iter()
                .map(|handle| handle.borrow().index())
                .collect(),
        )
    }

    /// Reindex children by the `indices`, e.g.
    ///
    /// ```plain
    /// indices = [3, 4, 1, 2]
    /// children => [children[3], children[4], children[1], children[2]]
    /// ```
    pub(super) fn reindex_children_of(
        &mut self,
        py: Python<'_>,
        root: &RelationHandle,
        indices: Vec<usize>,
    ) -> PyResult<()> {
        let node_index = root.index();
        let node = self.node_mut(node_index);
        let old = &node.children;

        if old.len() != indices.len() {
            return Err(ReindexError::new_err(t!("Indices count didn't match")));
        }

        let mut used: Vec<bool> = Vec::with_capacity(old.len());
        used.resize_with(old.len(), || false);

        let children = indices
            .into_iter()
            .map(|index| {
                if index >= old.len() {
                    return Err(ReindexError::new_err(t!("Index out of bound")));
                }
                if used[index] {
                    return Err(ReindexError::new_err(t!("Index duplicated")));
                }

                used[index] = true;
                let elem = &old[index];

                Ok((elem.0, elem.1.clone_ref(py)))
            })
            .collect::<PyResult<Vec<_>>>()?;

        node.children = children;
        self.children_changed(node_index)?;
        Ok(())
    }

    /// Clear parent objects
    pub(super) fn clear_parents_of(
        &mut self,
        py: Python<'_>,
        root: &RelationHandle,
    ) -> PyResult<()> {
        let index = root.index();

        let parent_indices: Vec<usize> = self
            .node(index)
            .parents
            .iter()
            .filter(|parent| upgrade_ref(py, &parent.1).is_some())
            .map(|parent| parent.0)
            .collect();

        for parent_index in parent_indices {
            self.remove_by_index(parent_index, &vec![index])?;
        }
        Ok(())
    }

    /// Clear child objects
    pub(super) fn clear_children_of(&mut self, root: &RelationHandle) -> PyResult<()> {
        let index = root.index();

        self.remove_by_index(
            index,
            &self
                .node(index)
                .children
                .iter()
                .map(|child| child.0)
                .collect(),
        )
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

    fn remove_by_index(&mut self, index: NodeIndex, children: &Vec<NodeIndex>) -> PyResult<()> {
        self.children_changed(index)?;

        for &child_index in children {
            let node = self.node_mut(index);
            if !node.has_child(child_index) {
                continue;
            }

            let Some(i) = node.children.iter().position(|x| x.0 == child_index) else {
                continue;
            };
            node.children.remove(i);

            let child_parents = &mut self.node_mut(child_index).parents;
            child_parents.remove(child_parents.iter().position(|x| x.0 == index).unwrap());

            self.parents_changed(child_index)?;
        }
        Ok(())
    }

    fn parents_changed(&self, index: NodeIndex) -> PyResult<()> {
        self.node(index).reset_ancestors();
        let descendants = self.descendant_set(index)?;
        for child in descendants.iter() {
            self.node(child).reset_ancestors();
        }
        Ok(())
    }

    fn children_changed(&self, index: NodeIndex) -> PyResult<()> {
        self.node(index).reset_descendants();
        let ancestors = self.ancestor_set(index)?;
        for child in ancestors.iter() {
            self.node(child).reset_descendants();
        }
        Ok(())
    }

    /// Get the ancestor bitset with cache,
    /// `Err` if the graph has cycle
    pub(super) fn ancestor_set(&self, node_index: NodeIndex) -> PyResult<Rc<OffsetBitSet>> {
        let node = self.node(node_index);
        self.cached_set(
            &node.ancestor_set,
            node.parents.iter().map(|parent| parent.0),
            Self::ancestor_set,
        )
    }

    /// Get the descendant bitset with cache,
    /// `Err` if the graph has cycle
    pub(super) fn descendant_set(&self, node_index: NodeIndex) -> PyResult<Rc<OffsetBitSet>> {
        let node = self.node(node_index);
        self.cached_set(
            &node.descendant_set,
            node.children.iter().map(|child| child.0),
            Self::descendant_set,
        )
    }

    /// `Err` if the graph has cycle
    fn cached_set(
        &self,
        cache: &RecursiveCache<OffsetBitSet>,
        neighbors: impl Iterator<Item = NodeIndex>,
        recurse: impl Fn(&Self, NodeIndex) -> PyResult<Rc<OffsetBitSet>>,
    ) -> PyResult<Rc<OffsetBitSet>> {
        cache.get_or_compute(
            || {
                let mut set = OffsetBitSet::new();

                for index in neighbors {
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
        self.cached_dfs(
            &node.ancestor_dfs,
            node.parents.iter().map(|parent| parent.0),
            Self::ancestor_dfs,
        )
    }

    /// Get the descendant vec with cache,
    /// `Err` if the graph has cycle
    pub(super) fn descendant_dfs(&self, node_index: NodeIndex) -> PyResult<Rc<Vec<NodeIndex>>> {
        let node = self.node(node_index);
        self.cached_dfs(
            &node.descendant_dfs,
            node.children.iter().map(|child| child.0),
            Self::descendant_dfs,
        )
    }

    /// `Err` if the graph has cycle
    fn cached_dfs(
        &self,
        cache: &RecursiveCache<Vec<NodeIndex>>,
        neighbors: impl Iterator<Item = NodeIndex>,
        recurse: impl Fn(&Self, NodeIndex) -> PyResult<Rc<Vec<NodeIndex>>>,
    ) -> PyResult<Rc<Vec<NodeIndex>>> {
        cache.get_or_compute(
            || {
                let mut res = Vec::new();

                for index in neighbors {
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

pub(super) struct Node {
    /// Used for checking whether a node is alive
    handle_ref: Py<PyWeakrefReference>,

    /// `parents` has the weak-reference to python object,
    /// because we allow the expiration of the parent object
    parents: Vec<(NodeIndex, Py<PyWeakrefReference>)>,
    /// `children` holds the reference to python object,
    /// because we want to keep the `Group` structure even if there is no reference outside
    children: Vec<(NodeIndex, Py<PyAny>)>,

    ancestor_set: RecursiveCache<OffsetBitSet>,
    descendant_set: RecursiveCache<OffsetBitSet>,

    ancestor_dfs: RecursiveCache<Vec<NodeIndex>>,
    descendant_dfs: RecursiveCache<Vec<NodeIndex>>,
}

impl Node {
    pub(super) fn new(handle: &Bound<'_, RelationHandle>) -> PyResult<Self> {
        Ok(Self {
            handle_ref: PyWeakrefReference::new(handle)?.unbind(),
            parents: Vec::new(),
            children: Vec::new(),
            ancestor_set: RecursiveCache::new(),
            descendant_set: RecursiveCache::new(),
            ancestor_dfs: RecursiveCache::new(),
            descendant_dfs: RecursiveCache::new(),
        })
    }

    pub(super) fn alive(&self, py: Python<'_>) -> bool {
        upgrade_ref(py, &self.handle_ref).is_some()
    }

    pub(super) fn cleanup(&mut self, py: Python<'_>) {
        // Based on the assumption described in the docstring of `RelationRegistry`,
        // - we can check parent's ref, it's the same as using `alive`
        // - we don't need to check children, because we have an reference to its related object
        self.parents
            .retain(|parent| upgrade_ref(py, &parent.1).is_some());

        self.reset_ancestors();
        self.reset_descendants();
    }

    fn has_child(&self, check: NodeIndex) -> bool {
        self.children.iter().any(|child| child.0 == check)
    }

    fn reset_ancestors(&self) {
        self.ancestor_set.reset();
        self.ancestor_dfs.reset();
    }

    fn reset_descendants(&self) {
        self.descendant_set.reset();
        self.descendant_dfs.reset();
    }

    pub(super) fn resolve_self<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyAny>> {
        let handle: PyRef<RelationHandle> = upgrade_ref(py, &self.handle_ref)?.extract().unwrap();
        handle.get_ref(py)
    }

    pub(super) fn resolve_parents<'py>(&self, py: Python<'py>) -> Vec<Bound<'py, PyAny>> {
        self.parents
            .iter()
            .filter_map(|parent| upgrade_ref(py, &parent.1))
            .collect()
    }

    pub(super) fn resolve_children(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        self.iter_child_refs()
            .map(|child| child.clone_ref(py))
            .collect()
    }

    #[inline]
    pub(super) fn iter_child_refs(&self) -> impl Iterator<Item = &Py<PyAny>> + '_ {
        self.children.iter().map(|child| &child.1)
    }

    #[inline]
    pub(super) fn len(&self) -> usize {
        self.children.len()
    }
}
