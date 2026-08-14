use pyo3::exceptions::PyException;
use pyo3::{IntoPyObjectExt, prelude::*};

use janim_backend::bridge;
use janim_backend::gui;
use janim_backend::janim_backend as py_janim_backend;

fn main() {
    pyo3::append_to_inittab!(py_janim_backend);

    Python::initialize();
    Python::attach(|py| {
        let ret = app(py);
        match ret {
            Ok(_) => (),
            Err(err) => err.print(py),
        }
    });
}

fn app(py: Python<'_>) -> PyResult<()> {
    let module = PyModule::from_code(
        py,
        cr#"
from janim_backend import bridge

def make_timeline_labels(built: BuiltTimeline) -> bridge.TimelineTimeLabels:
    return bridge.TimelineTimeLabels([], [], [], [])

def extract_information(built) -> bridge.ExtractedTimeline:
    return bridge.ExtractedTimeline(
        'placeholder',
        10,
        make_timeline_labels(built),
    )
            "#,
        c"defs.py",
        c"defs",
    )?;

    let extract_information = module.getattr("extract_information")?;

    let args = bridge::AppArgs {
        callbacks: bridge::Callbacks {
            extract_information: extract_information.extract()?,
        },
        setup_built_timelines: vec![().into_py_any(py)?],
    };
    gui::run(args).map_err(|err| PyException::new_err(err.to_string()))
}
