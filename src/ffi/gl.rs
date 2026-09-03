use std::ffi::{CStr, c_char, c_void};
use std::sync::OnceLock;

use gl33::{GLenum, GlFns};

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

#[inline]
fn error_name(error: u32) -> &'static str {
    match error {
        Gl::GL_INVALID_ENUM => "GL_INVALID_ENUM",
        Gl::GL_INVALID_VALUE => "GL_INVALID_VALUE",
        Gl::GL_INVALID_OPERATION => "GL_INVALID_OPERATION",
        Gl::GL_INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
        Gl::GL_OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
        _ => "UNKNOWN",
    }
}

#[pyclass(module = "janim_backend.ffi", name = "gl", unsendable)]
pub struct Gl;

static GL: OnceLock<GlFns> = OnceLock::new();

impl Gl {
    #[inline]
    fn get_gl() -> PyResult<&'static GlFns> {
        GL.get()
            .ok_or_else(|| PyRuntimeError::new_err("OpenGL is not loaded; call `gl.load()` first"))
    }

    /// Check the OpenGL error state after a call.
    ///
    /// OpenGL errors are sticky, so this deliberately checks only the state
    /// produced by the preceding operation. The caller should not make other
    /// GL calls between the operation and this function.
    #[inline]
    fn check_error(function: &'static str) -> PyResult<()> {
        let error = unsafe { Self::get_gl()?.GetError() };
        let error_code = error.0;

        if error_code != Gl::GL_NO_ERROR {
            return Err(PyRuntimeError::new_err(format!(
                "{function} failed: {} (0x{error_code:04X})",
                error_name(error_code),
            )));
        }

        Ok(())
    }
}

#[pymethods]
impl Gl {
    // -----------------------------------------------------------------------
    // OpenGL 3.x constants used by JAnim.
    // -----------------------------------------------------------------------

    #[classattr]
    const GL_NO_ERROR: u32 = gl33::GL_NO_ERROR.0;

    #[classattr]
    const GL_INVALID_ENUM: u32 = gl33::GL_INVALID_ENUM.0;
    #[classattr]
    const GL_INVALID_VALUE: u32 = gl33::GL_INVALID_VALUE.0;
    #[classattr]
    const GL_INVALID_OPERATION: u32 = gl33::GL_INVALID_OPERATION.0;
    #[classattr]
    const GL_OUT_OF_MEMORY: u32 = gl33::GL_OUT_OF_MEMORY.0;
    #[classattr]
    const GL_INVALID_FRAMEBUFFER_OPERATION: u32 = gl33::GL_INVALID_FRAMEBUFFER_OPERATION.0;

    #[classattr]
    const GL_TEXTURE0: u32 = gl33::GL_TEXTURE0.0;
    #[classattr]
    const GL_TEXTURE1: u32 = gl33::GL_TEXTURE1.0;
    #[classattr]
    const GL_TEXTURE2: u32 = gl33::GL_TEXTURE2.0;
    #[classattr]
    const GL_TEXTURE3: u32 = gl33::GL_TEXTURE3.0;
    #[classattr]
    const GL_TEXTURE4: u32 = gl33::GL_TEXTURE4.0;
    #[classattr]
    const GL_TEXTURE5: u32 = gl33::GL_TEXTURE5.0;
    #[classattr]
    const GL_TEXTURE6: u32 = gl33::GL_TEXTURE6.0;
    #[classattr]
    const GL_TEXTURE7: u32 = gl33::GL_TEXTURE7.0;

    #[classattr]
    const GL_TEXTURE_BUFFER: u32 = gl33::GL_TEXTURE_BUFFER.0;
    #[classattr]
    const GL_RGBA32F: u32 = gl33::GL_RGBA32F.0;

    #[classattr]
    const GL_PIXEL_PACK_BUFFER: u32 = gl33::GL_PIXEL_PACK_BUFFER.0;
    #[classattr]
    const GL_STREAM_READ: u32 = gl33::GL_STREAM_READ.0;

    #[classattr]
    const GL_READ_ONLY: u32 = gl33::GL_READ_ONLY.0;

    #[classattr]
    const GL_RGBA: u32 = gl33::GL_RGBA.0;
    #[classattr]
    const GL_UNSIGNED_BYTE: u32 = gl33::GL_UNSIGNED_BYTE.0;

    /// Load OpenGL native functions.
    #[staticmethod]
    fn load() -> PyResult<()> {
        gl_loader::init_gl();

        let loader = |name: *const u8| -> *const c_void {
            let name = unsafe { CStr::from_ptr(name as *const c_char) };

            let name = match name.to_str() {
                Ok(name) => name,
                Err(_) => return std::ptr::null(),
            };

            let address = gl_loader::get_proc_address(name);
            address as *const c_void
        };

        let gl = unsafe { GlFns::load_from(&loader) }.map_err(|name| {
            PyRuntimeError::new_err(format!("failed to load OpenGL function: {name}"))
        })?;

        GL.set(gl)
            .map_err(|_| PyRuntimeError::new_err("OpenGL has already been loaded"))
    }

    /// Return whether OpenGL has been loaded.
    #[staticmethod]
    fn is_loaded() -> bool {
        GL.get().is_some()
    }

    // -----------------------------------------------------------------------
    // Texture
    // -----------------------------------------------------------------------

    /// glGenTextures
    #[pyo3(name = "glGenTextures")]
    #[staticmethod]
    fn gen_textures(n: usize) -> PyResult<Vec<u32>> {
        let n = i32::try_from(n).expect("number of textures is too large");
        let mut textures = vec![0u32; n as usize];

        unsafe {
            Self::get_gl()?.GenTextures(n, textures.as_mut_ptr());
        }
        Self::check_error("glGenTextures")?;

        Ok(textures)
    }

    /// glBindTexture
    #[pyo3(name = "glBindTexture")]
    #[staticmethod]
    fn bind_texture(target: u32, texture: u32) -> PyResult<()> {
        unsafe {
            Self::get_gl()?.BindTexture(GLenum(target), texture);
        }
        Self::check_error("glBindTexture")
    }

    /// glTexBuffer
    #[pyo3(name = "glTexBuffer")]
    #[staticmethod]
    fn tex_buffer(target: u32, internalformat: u32, buffer: u32) -> PyResult<()> {
        unsafe {
            Self::get_gl()?.TexBuffer(GLenum(target), GLenum(internalformat), buffer);
        }
        Self::check_error("glTexBuffer")
    }

    /// glActiveTexture
    #[pyo3(name = "glActiveTexture")]
    #[staticmethod]
    fn active_texture(texture: u32) -> PyResult<()> {
        unsafe {
            Self::get_gl()?.ActiveTexture(GLenum(texture));
        }
        Self::check_error("glActiveTexture")
    }

    // -----------------------------------------------------------------------
    // Buffer
    // -----------------------------------------------------------------------

    /// glGenBuffers
    #[pyo3(name = "glGenBuffers")]
    #[staticmethod]
    fn gen_buffers(n: usize) -> PyResult<Vec<u32>> {
        let n = i32::try_from(n).expect("number of buffers is too large");
        let mut buffers = vec![0u32; n as usize];

        unsafe {
            Self::get_gl()?.GenBuffers(n, buffers.as_mut_ptr());
        }
        Self::check_error("glGenBuffers")?;

        Ok(buffers)
    }

    /// glBindBuffer
    #[pyo3(name = "glBindBuffer")]
    #[staticmethod]
    fn bind_buffer(target: u32, buffer: u32) -> PyResult<()> {
        unsafe {
            Self::get_gl()?.BindBuffer(GLenum(target), buffer);
        }
        Self::check_error("glBindBuffer")
    }

    /// glBufferData
    ///
    /// `data=None` allocates uninitialized GPU storage.
    #[pyo3(name = "glBufferData")]
    #[staticmethod]
    fn buffer_data(target: u32, size: isize, data: Option<Py<PyAny>>, usage: u32) -> PyResult<()> {
        let ptr = match data {
            None => std::ptr::null(),
            Some(_) => {
                panic!("`glBufferData` with non-None `data` is not supported");
            }
        };

        unsafe {
            Self::get_gl()?.BufferData(GLenum(target), size, ptr, GLenum(usage));
        }
        Self::check_error("glBufferData")
    }

    /// glDeleteBuffers
    #[pyo3(name = "glDeleteBuffers")]
    #[staticmethod]
    fn delete_buffers(buffers: Vec<u32>) -> PyResult<()> {
        if buffers.is_empty() {
            return Ok(());
        }
        let n = i32::try_from(buffers.len()).expect("too many buffers");

        unsafe {
            Self::get_gl()?.DeleteBuffers(n, buffers.as_ptr());
        }
        Self::check_error("glDeleteBuffers")
    }

    /// glMapBuffer
    ///
    /// Returns the mapped address as an integer.
    ///
    /// The caller must call glUnmapBuffer before the buffer is rebound or
    /// otherwise invalidated.
    #[pyo3(name = "glMapBuffer")]
    #[staticmethod]
    fn map_buffer(target: u32, access: u32) -> PyResult<usize> {
        let ptr = unsafe { Self::get_gl()?.MapBuffer(GLenum(target), GLenum(access)) };
        Self::check_error("glMapBuffer")?;

        if ptr.is_null() {
            return Err(PyRuntimeError::new_err("`glMapBuffer` returned `NULL`"));
        }
        Ok(ptr as usize)
    }

    /// glUnmapBuffer
    ///
    /// Returns `False` if the contents of the mapped buffer became corrupt.
    #[pyo3(name = "glUnmapBuffer")]
    #[staticmethod]
    fn unmap_buffer(target: u32) -> PyResult<bool> {
        let result = unsafe { Self::get_gl()?.UnmapBuffer(GLenum(target)) };
        Self::check_error("glUnmapBuffer")?;

        Ok(result != 0)
    }

    /// glGetBufferSubData
    ///
    /// Returns a bytes object containing the requested GPU buffer contents.
    #[pyo3(name = "glGetBufferSubData")]
    #[staticmethod]
    fn get_buffer_sub_data<'py>(
        py: Python<'py>,
        target: u32,
        offset: isize,
        size: usize,
    ) -> PyResult<Bound<'py, PyBytes>> {
        let size = isize::try_from(size).expect("size is too large");
        let mut data = vec![0u8; size as usize];

        unsafe {
            Self::get_gl()?.GetBufferSubData(
                GLenum(target),
                offset,
                size,
                data.as_mut_ptr() as *mut c_void,
            );
        }
        Self::check_error("glGetBufferSubData")?;

        Ok(PyBytes::new(py, &data))
    }

    // -----------------------------------------------------------------------
    // Program / Uniform
    // -----------------------------------------------------------------------

    /// glUseProgram
    #[pyo3(name = "glUseProgram")]
    #[staticmethod]
    fn use_program(program: u32) -> PyResult<()> {
        Self::get_gl()?.UseProgram(program);
        Self::check_error("glUseProgram")
    }

    /// glGetUniformLocation
    #[pyo3(name = "glGetUniformLocation")]
    #[staticmethod]
    fn get_uniform_location(program: u32, name: &str) -> PyResult<i32> {
        let name = std::ffi::CString::new(name)
            .map_err(|_| PyRuntimeError::new_err("uniform name contains `NUL` byte"))?;

        let location =
            unsafe { Self::get_gl()?.GetUniformLocation(program, name.as_ptr() as *const u8) };
        // -1 is a valid return value when the uniform is not active.
        Self::check_error("glGetUniformLocation")?;

        Ok(location)
    }

    /// glUniform1i
    #[pyo3(name = "glUniform1i")]
    #[staticmethod]
    fn uniform_1i(location: i32, value: i32) -> PyResult<()> {
        unsafe {
            Self::get_gl()?.Uniform1i(location, value);
        }
        Self::check_error("glUniform1i")
    }

    // -----------------------------------------------------------------------
    // Pixel readback
    // -----------------------------------------------------------------------

    /// glReadPixels
    ///
    /// `pixels` is interpreted exactly like OpenGL:
    ///
    /// - if `GL_PIXEL_PACK_BUFFER` is not bound, it is a host pointer;
    /// - if `GL_PIXEL_PACK_BUFFER` is bound, it is a byte offset into the PBO.
    ///
    /// For the PBO use case in JAnim, pass `0`.
    #[pyo3(name = "glReadPixels")]
    #[allow(clippy::too_many_arguments)]
    #[staticmethod]
    fn read_pixels(
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        type_: u32,
        pixels: usize,
    ) -> PyResult<()> {
        unsafe {
            Self::get_gl()?.ReadPixels(
                x,
                y,
                width,
                height,
                GLenum(format),
                GLenum(type_),
                pixels as *mut c_void,
            );
        }
        Self::check_error("glReadPixels")
    }
}
