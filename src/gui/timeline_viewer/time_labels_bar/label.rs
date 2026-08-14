use std::cell::{Cell, Ref, RefCell};

use super::paint::Color;

pub type LabelId = usize;

pub(super) const LABEL_DEFAULT_HEIGHT: i32 = 3;
pub(super) const LABEL_GROUP_HEADER_COLLAPSED_HEIGHT: i32 = 3;
pub(super) const LABEL_GROUP_HEADER_EXPANDED_HEIGHT: i32 = 2;

pub(super) const LABEL_PIXEL_HEIGHT_PER_UNIT: f32 = 8.0; // px

#[derive(Default)]
pub struct LabelLayout {
    nodes: Vec<LabelNode>,
}

impl LabelLayout {
    // ==== Label Creation ====

    pub fn create_label(&mut self, info: LabelInfo, height: Option<i32>) -> LabelId {
        let id = self.nodes.len();
        self.nodes.push(LabelNode::new(
            info,
            LabelItemData::new(height.unwrap_or(LABEL_DEFAULT_HEIGHT)),
        ));
        id
    }

    pub fn create_group(
        &mut self,
        info: LabelInfo,
        mut children_id: Vec<LabelId>,
        collapsed: bool,
        header: bool,
        highlight: Color,
    ) -> LabelId {
        let id = self.nodes.len();

        // Sort children by range start point
        // We simulate the process of stacking labels in this order
        children_id.sort_by(|a, b| self.node(*a).at().total_cmp(&self.node(*b).end()));

        // The optimization for iterating, we only enable it when there are more than 32 child-labels
        // See `LabelGroupData::divisions` for details
        let mut layers: Option<Vec<Vec<LabelId>>> = if children_id.len() > 32 {
            Some(Vec::new())
        } else {
            None
        };

        // Used for simulate the process of stacking labels
        // The index in `stack` indicates the layer where the label is placed
        //
        // Note: the y-axis is top-to-bottom,
        // so we use "above" to refer to lower-index layers and "below" to refer to higher-index layers
        let mut stack: Vec<LabelId> = Vec::new();

        // Iterate child-labels, simulate the process of stacking labels
        for child_id in &children_id {
            self.node_mut(*child_id).state.parent = Some(id);

            // Iterate the layers, decide to place to which layer, and find the last layer that blocking the current label
            let mut found_place = false;
            let mut last_layer = 0;
            for i in 0..stack.len() {
                let other_id = stack[i];

                // If the current layer does not block the current label, we can place the label in if not already placed
                //
                // + 1e-5 to avoid floating-inprecision for too-close labels
                if self.node(other_id).end() <= self.node(*child_id).at() + 1e-5 {
                    if !found_place {
                        if let Some(layers) = layers.as_mut() {
                            layers[i].push(*child_id);
                        }
                        stack[i] = *child_id;
                        found_place = true;
                        // Becasue after placing this label, the current layer is blocked by it again,
                        // so we also needs to set `last_layer` here
                        last_layer = i;
                    }
                } else {
                    last_layer = i;

                    // Mark `above_labels` and `below_labels` between child-labels
                    if found_place {
                        self.node_mut(other_id).state.below_labels.push(*child_id);
                        self.node_mut(*child_id).state.above_labels.push(other_id);
                    } else {
                        self.node_mut(*child_id).state.below_labels.push(other_id);
                        self.node_mut(other_id).state.above_labels.push(*child_id);
                    }
                }
            }

            // Only if `found_place`, we try to remove the stacking layers after the `last_layer`
            //
            // If not `found_place`, we place the label in the new layer after the stacking layers
            if found_place {
                if last_layer != stack.len() {
                    stack.truncate(last_layer + 1);
                }
            } else {
                if let Some(layers) = layers.as_mut() {
                    if stack.len() == layers.len() {
                        layers.push(vec![*child_id]);
                    } else {
                        layers[stack.len()].push(*child_id);
                    }
                }

                stack.push(*child_id);
            }
        }

        self.nodes.push(LabelNode::new(
            info,
            LabelGroupData::new(children_id, layers, collapsed, header, highlight),
        ));
        id
    }

    // ==== Label Layout Calculation ====

    fn switch_collapse(&mut self, id: LabelId) {
        match &mut self.node_mut(id).data {
            LabelData::LabelItem(_) => unreachable!(),
            LabelData::LabelGroup(data) => data.collapsed = !data.collapsed,
        };
        let LabelData::LabelGroup(data) = &mut self.node_mut(id).data else {
            unreachable!();
        };
        data.collapsed = !data.collapsed;

        self.mark_needs_refresh_height(id);
    }

    fn mark_needs_refresh_height(&self, id: LabelId) {
        let node = self.node(id);
        match &node.data {
            LabelData::LabelItem(_) => unreachable!(),
            LabelData::LabelGroup(data) => data.height.set(None),
        };

        for other in self.all_below_labels(id).iter() {
            match &self.node(*other).data {
                LabelData::LabelItem(_) => unreachable!(),
                LabelData::LabelGroup(data) => data.height.set(None),
            };
        }

        if let Some(parent) = &node.state.parent {
            self.mark_needs_refresh_height(*parent);
        }
    }

    pub(super) fn node_y(&self, id: LabelId) -> i32 {
        let state = &self.node(id).state;
        match state.y.get() {
            Some(y) => y,
            None => {
                let y = state
                    .above_labels
                    .iter()
                    .map(|label_id| self.node_y(*label_id) + self.node_height(*label_id))
                    .max()
                    .unwrap_or(0);
                state.y.set(Some(y));
                y
            }
        }
    }

    fn node_height(&self, id: LabelId) -> i32 {
        match &self.node(id).data {
            LabelData::LabelItem(data) => data.height,
            LabelData::LabelGroup(data) => match data.height.get() {
                Some(height) => height,
                None => {
                    let content_height = data
                        .children
                        .iter()
                        .map(|label_id| self.node_y(*label_id) + self.node_height(*label_id))
                        .max()
                        .unwrap_or(0);
                    let height = data.header_height() + content_height;
                    data.height.set(Some(height));
                    height
                }
            },
        }
    }

    // ==== Utils ====

    pub(super) fn node(&self, id: LabelId) -> &LabelNode {
        &self.nodes[id]
    }

    fn node_mut(&mut self, id: LabelId) -> &mut LabelNode {
        &mut self.nodes[id]
    }

    fn all_below_labels(&self, id: LabelId) -> Ref<'_, Vec<LabelId>> {
        let state = &self.node(id).state;

        if state.all_below_labels.borrow().is_none() {
            let mut result = Vec::new();

            for label_id in &state.below_labels {
                if !result.contains(label_id) {
                    result.push(*label_id);
                }
                for x in self.all_below_labels(*label_id).iter() {
                    if !result.contains(x) {
                        result.push(*x);
                    }
                }
            }

            *state.all_below_labels.borrow_mut() = Some(result);
        }

        Ref::map(state.all_below_labels.borrow(), |x| x.as_ref().unwrap())
    }
}

pub(super) struct LabelNode {
    pub(super) info: LabelInfo,
    pub(super) state: LabelState,
    pub(super) data: LabelData,
}

impl LabelNode {
    pub fn new(info: LabelInfo, data: LabelData) -> Self {
        Self {
            info,
            state: LabelState::default(),
            data,
        }
    }

    pub fn at(&self) -> f32 {
        self.info.range.0
    }
    pub fn end(&self) -> f32 {
        self.info.range.1
    }
}

pub struct LabelInfo {
    pub text: Option<String>,
    pub color: Color,
    pub range: (f32, f32),
}

#[derive(Default)]
struct LabelState {
    parent: Option<LabelId>,
    /// Which labels are be placed above this label,
    /// excluding nested labels.
    above_labels: Vec<LabelId>,
    /// Which labels are be placed below this label,
    /// excluding nested labels.
    below_labels: Vec<LabelId>,
    /// All labels below below this label, including nested labels.
    /// Used for marking all of them needs to compute the new [LabelState::y]
    ///
    /// This value is lazy-evaluated.
    all_below_labels: RefCell<Option<Vec<LabelId>>>,

    /// `None` indicates needs refresh
    y: Cell<Option<i32>>,
}

pub(super) enum LabelData {
    LabelItem(LabelItemData),
    LabelGroup(LabelGroupData),
}

pub(super) struct LabelItemData {
    pub height: i32,
}

impl LabelItemData {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(height: i32) -> LabelData {
        LabelData::LabelItem(Self { height })
    }
}

pub(super) struct LabelGroupData {
    pub(super) children: Vec<LabelId>,
    /// Used for optimize the query for child-labels in a range,
    /// labels (the inner `Vec<LabelId>`) here are placed in layers (the outer `Vec`).
    ///
    /// Each label in the same layer has no overlaps,
    /// so we can use binary search for querying them fastly.
    pub(super) layers: Option<Vec<Vec<LabelId>>>,

    /// Whether this label group is collapsed, affecting the content display and label height
    pub(super) collapsed: bool,
    /// Whether the header is visible
    pub(super) header: bool,

    /// `None` indicates needs refresh
    height: Cell<Option<i32>>,

    pub(super) highlight: Color,
}

impl LabelGroupData {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        children: Vec<LabelId>,
        layers: Option<Vec<Vec<LabelId>>>,
        collapsed: bool,
        header: bool,
        highlight: Color,
    ) -> LabelData {
        LabelData::LabelGroup(Self {
            children,
            layers,
            highlight,
            collapsed,
            header,
            height: Cell::new(None),
        })
    }

    pub fn header_height(&self) -> i32 {
        if self.collapsed {
            LABEL_GROUP_HEADER_COLLAPSED_HEIGHT
        } else {
            LABEL_GROUP_HEADER_EXPANDED_HEIGHT
        }
    }
}
