use pyo3::{prelude::*, types::PyFunction};

crate::resolve_all! {
    module = "janim_backend.bridge";

    pub struct TimelineTimeLabels {
        #[pyresolver]
        pub subtimeline_labels: Vec<SubtimelineTimeLabel>,
        #[pyresolver]
        pub debug_labels: Vec<DebugTimeLabel>,
        #[pyresolver]
        pub audio_labels: Vec<AudioTimeLabel>,
        #[pyresolver]
        pub anim_labels: Vec<AnimTimeLabel>,
    }

    // ---- Debug ----

    pub struct DebugTimeLabel {
        pub item_repr: String,
        pub visibility: Vec<f32>,
        #[pyresolver]
        pub chunks: Vec<AnimChunk>
    }

    pub struct AnimChunk {
        pub start: f32,
        /// (anim_id: i32, label_desc: String)
        pub list: Vec<(u64, String)>,
    }

    // ---- Audio ----

    pub struct AudioTimeLabel {
        pub range: (f32, f32),
        pub file_name: String
    }

    // ---- Anim ----

    pub enum AnimTimeLabel {
        AnimGroup(#[pyresolver]AnimBaseInfo, #[pyresolver]Vec<AnimTimeLabel>),
        Animation(#[pyresolver]AnimBaseInfo, f32, Option<f32>),
    }

    pub struct AnimBaseInfo {
        pub name: String,
        pub color_rgb: (u8, u8, u8),
    }

    // ---- Subtimeline ----

    pub struct SubtimelineTimeLabel {
        pub label_desc: String,
        pub range: (f32, f32),
        pub first_frame_duration: f32,
        // () -> TimelineTimeLabels
        pub lazy_setup: Py<PyFunction>
    }
}
