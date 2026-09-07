use std::ffi::{CStr, c_char, c_void};
use std::sync::OnceLock;

use gl33::{GLenum, GlFns};

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use super::gl;

static GL: OnceLock<GlFns> = OnceLock::new();

#[inline]
pub(super) fn get_gl() -> PyResult<&'static GlFns> {
    GL.get()
        .ok_or_else(|| PyRuntimeError::new_err("OpenGL is not loaded; call `gl.load()` first"))
}

#[inline]
fn error_name(error: u32) -> &'static str {
    match error {
        gl::GL_INVALID_ENUM => "GL_INVALID_ENUM",
        gl::GL_INVALID_VALUE => "GL_INVALID_VALUE",
        gl::GL_INVALID_OPERATION => "GL_INVALID_OPERATION",
        gl::GL_INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
        gl::GL_OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
        _ => "UNKNOWN",
    }
}

/// Check the OpenGL error state after a call.
///
/// OpenGL errors are sticky, so this deliberately checks only the state
/// produced by the preceding operation. The caller should not make other
/// GL calls between the operation and this function.
#[inline]
fn check_error(function: &'static str) -> PyResult<()> {
    let error = unsafe { get_gl()?.GetError() };
    let error_code = error.0;

    if error_code != gl::GL_NO_ERROR {
        return Err(PyRuntimeError::new_err(format!(
            "{function} failed: {} (0x{error_code:04X})",
            error_name(error_code),
        )));
    }

    Ok(())
}

/// Load OpenGL native functions.
#[pyfunction]
pub fn load() -> PyResult<()> {
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
#[pyfunction]
pub fn is_loaded() -> bool {
    GL.get().is_some()
}

// -----------------------------------------------------------------------
// Texture
// -----------------------------------------------------------------------

/// glGenTextures
#[pyfunction(name = "glGenTextures")]
pub fn gen_textures(n: usize) -> PyResult<Vec<u32>> {
    let n = i32::try_from(n).expect("number of textures is too large");
    let mut textures = vec![0u32; n as usize];

    unsafe {
        get_gl()?.GenTextures(n, textures.as_mut_ptr());
    }
    check_error("glGenTextures")?;

    Ok(textures)
}

/// glBindTexture
#[pyfunction(name = "glBindTexture")]
pub fn bind_texture(target: u32, texture: u32) -> PyResult<()> {
    unsafe {
        get_gl()?.BindTexture(GLenum(target), texture);
    }
    check_error("glBindTexture")
}

/// glTexBuffer
#[pyfunction(name = "glTexBuffer")]
pub fn tex_buffer(target: u32, internalformat: u32, buffer: u32) -> PyResult<()> {
    unsafe {
        get_gl()?.TexBuffer(GLenum(target), GLenum(internalformat), buffer);
    }
    check_error("glTexBuffer")
}

/// glActiveTexture
#[pyfunction(name = "glActiveTexture")]
pub fn active_texture(texture: u32) -> PyResult<()> {
    unsafe {
        get_gl()?.ActiveTexture(GLenum(texture));
    }
    check_error("glActiveTexture")
}

// -----------------------------------------------------------------------
// Buffer
// -----------------------------------------------------------------------

/// glGenBuffers
#[pyfunction(name = "glGenBuffers")]
pub fn gen_buffers(n: usize) -> PyResult<Vec<u32>> {
    let n = i32::try_from(n).expect("number of buffers is too large");
    let mut buffers = vec![0u32; n as usize];

    unsafe {
        get_gl()?.GenBuffers(n, buffers.as_mut_ptr());
    }
    check_error("glGenBuffers")?;

    Ok(buffers)
}

/// glBindBuffer
#[pyfunction(name = "glBindBuffer")]
pub fn bind_buffer(target: u32, buffer: u32) -> PyResult<()> {
    unsafe {
        get_gl()?.BindBuffer(GLenum(target), buffer);
    }
    check_error("glBindBuffer")
}

/// glBufferData
///
/// `data=None` allocates uninitialized GPU storage.
#[pyfunction(name = "glBufferData")]
pub fn buffer_data(target: u32, size: isize, data: Option<Py<PyAny>>, usage: u32) -> PyResult<()> {
    let ptr = match data {
        None => std::ptr::null(),
        Some(_) => {
            panic!("`glBufferData` with non-None `data` is not supported");
        }
    };

    unsafe {
        get_gl()?.BufferData(GLenum(target), size, ptr, GLenum(usage));
    }
    check_error("glBufferData")
}

/// glDeleteBuffers
#[pyfunction(name = "glDeleteBuffers")]
pub fn delete_buffers(buffers: Vec<u32>) -> PyResult<()> {
    if buffers.is_empty() {
        return Ok(());
    }
    let n = i32::try_from(buffers.len()).expect("too many buffers");

    unsafe {
        get_gl()?.DeleteBuffers(n, buffers.as_ptr());
    }
    check_error("glDeleteBuffers")
}

/// glMapBuffer
///
/// Returns the mapped address as an integer.
///
/// The caller must call glUnmapBuffer before the buffer is rebound or
/// otherwise invalidated.
#[pyfunction(name = "glMapBuffer")]
pub fn map_buffer(target: u32, access: u32) -> PyResult<usize> {
    let ptr = unsafe { get_gl()?.MapBuffer(GLenum(target), GLenum(access)) };
    check_error("glMapBuffer")?;

    if ptr.is_null() {
        return Err(PyRuntimeError::new_err("`glMapBuffer` returned `NULL`"));
    }
    Ok(ptr as usize)
}

/// glUnmapBuffer
///
/// Returns `False` if the contents of the mapped buffer became corrupt.
#[pyfunction(name = "glUnmapBuffer")]
pub fn unmap_buffer(target: u32) -> PyResult<bool> {
    let result = unsafe { get_gl()?.UnmapBuffer(GLenum(target)) };
    check_error("glUnmapBuffer")?;

    Ok(result != 0)
}

/// glGetBufferSubData
///
/// Returns a bytes object containing the requested GPU buffer contents.
#[pyfunction(name = "glGetBufferSubData")]
pub fn get_buffer_sub_data<'py>(
    py: Python<'py>,
    target: u32,
    offset: isize,
    size: usize,
) -> PyResult<Bound<'py, PyBytes>> {
    let size = isize::try_from(size).expect("size is too large");
    let mut data = vec![0u8; size as usize];

    unsafe {
        get_gl()?.GetBufferSubData(
            GLenum(target),
            offset,
            size,
            data.as_mut_ptr() as *mut c_void,
        );
    }
    check_error("glGetBufferSubData")?;

    Ok(PyBytes::new(py, &data))
}

// -----------------------------------------------------------------------
// Program / Uniform
// -----------------------------------------------------------------------

/// glUseProgram
#[pyfunction(name = "glUseProgram")]
pub fn use_program(program: u32) -> PyResult<()> {
    get_gl()?.UseProgram(program);
    check_error("glUseProgram")
}

/// glGetUniformLocation
#[pyfunction(name = "glGetUniformLocation")]
pub fn get_uniform_location(program: u32, name: &str) -> PyResult<i32> {
    let name = std::ffi::CString::new(name)
        .map_err(|_| PyRuntimeError::new_err("uniform name contains `NUL` byte"))?;

    let location = unsafe { get_gl()?.GetUniformLocation(program, name.as_ptr() as *const u8) };
    // -1 is a valid return value when the uniform is not active.
    check_error("glGetUniformLocation")?;

    Ok(location)
}

/// glUniform1i
#[pyfunction(name = "glUniform1i")]
pub fn uniform_1i(location: i32, value: i32) -> PyResult<()> {
    unsafe {
        get_gl()?.Uniform1i(location, value);
    }
    check_error("glUniform1i")
}

#[pyfunction(name = "glUniform1f")]
pub fn uniform_1f(location: i32, value: f32) -> PyResult<()> {
    unsafe {
        get_gl()?.Uniform1f(location, value);
    }
    check_error("glUniform1f")
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
#[allow(clippy::too_many_arguments)]
#[pyfunction(name = "glReadPixels")]
pub fn read_pixels(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    format: u32,
    type_: u32,
    pixels: usize,
) -> PyResult<()> {
    unsafe {
        get_gl()?.ReadPixels(
            x,
            y,
            width,
            height,
            GLenum(format),
            GLenum(type_),
            pixels as *mut c_void,
        );
    }
    check_error("glReadPixels")
}
