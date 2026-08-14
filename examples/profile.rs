use pyo3::prelude::*;

// use janim_backend::janim_backend::exec;

/// Commands:
/// - `cargo build --example profile --profile profiling`
/// - `samply record ./target/profiling/examples/profile`
fn main() {
    // println!("Hello");
    Python::initialize();
    Python::attach(|py| {
        // let _ = exec();
    });
}
