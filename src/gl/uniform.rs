use pyo3::{prelude::*, types::PyBytes};

use super::gl;

/// A fast version of moderngl's `Uniform` class.
///
/// Compared to moderngl's `Uniform` class:
///
/// - Only supports writing single-element uniforms; array-shaped uniforms are
///   not supported.
/// - Does not support reading stored uniform values.
/// - Requires the uniform type to be specified manually.
#[pyclass(module = "janim_backend.gl")]
pub struct FastUniform {
    prog_glo: u32,
    location: i32,
    gl_type: u32,
}

impl FastUniform {
    #[inline]
    fn use_program(&self) -> PyResult<()> {
        gl::use_program(self.prog_glo)
    }
}

#[pymethods]
impl FastUniform {
    #[new]
    fn new(prog_glo: u32, name: &str, gl_type: u32) -> PyResult<Self> {
        Ok(Self {
            prog_glo,
            location: gl::get_uniform_location(prog_glo, name)?,
            gl_type,
        })
    }

    fn write_bool(&self, value: bool) -> PyResult<()> {
        let v = if value { 1 } else { 0 };
        self.use_program()?;
        gl::uniform_1i(self.location, v)
    }

    fn write_int(&self, value: i32) -> PyResult<()> {
        self.use_program()?;
        gl::uniform_1i(self.location, value)
    }

    fn write_float(&self, value: f32) -> PyResult<()> {
        self.use_program()?;
        gl::uniform_1f(self.location, value)
    }

    fn write_vec2(&self, v0: f32, v1: f32) -> PyResult<()> {
        self.use_program()?;
        gl::uniform_2f(self.location, v0, v1)
    }

    fn write_bytes(&self, value: Bound<'_, PyBytes>) -> PyResult<()> {
        self.use_program()?;
        gl::uniform_bytes(self.location, value, self.gl_type)
    }
}
