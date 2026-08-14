use pyo3::{prelude::*, types::PyFunction};
use pyresolver::PyResolver;

use crate::bridge::time_labels::{PyTimelineTimeLabels, TimelineTimeLabels};

pub mod time_labels;

#[derive(Debug, PyResolver)]
#[pyresolver(module = "janim_backend.bridge")]
pub struct Callbacks {
    /// (BuiltTimeline) -> ExtractedTimeline
    pub extract_information: Py<PyFunction>,
}

#[derive(Debug, PyResolver)]
#[pyresolver(module = "janim_backend.bridge")]
pub struct AppArgs {
    #[pyresolver]
    pub callbacks: Callbacks,
    pub setup_built_timelines: Vec<Py<PyAny>>,
}

#[derive(Debug, PyResolver)]
#[pyresolver(module = "janim_backend.bridge")]
pub struct ExtractedTimeline {
    pub timeline_name: String,
    pub duration: f32,
    #[pyresolver]
    pub time_labels: TimelineTimeLabels,
}

#[pymodule]
pub mod bridge {
    #[pymodule_export]
    use super::time_labels::{
        PyAnimBaseInfo, PyAnimChunk, PyAnimTimeLabel, PyAudioTimeLabel, PyDebugTimeLabel,
        PySubtimelineTimeLabel, PyTimelineTimeLabels,
    };

    #[pymodule_export]
    use super::{PyAppArgs, PyCallbacks, PyExtractedTimeline};
}
