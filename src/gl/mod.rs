mod functions;
mod uniform;

use pyo3::prelude::*;

#[pymodule]
pub mod gl {
    // -----------------------------------------------------------------------
    // OpenGL 3.x constants used by JAnim.
    // -----------------------------------------------------------------------

    #[pymodule_export]
    pub const GL_NO_ERROR: u32 = gl33::GL_NO_ERROR.0;

    #[pymodule_export]
    pub const GL_INVALID_ENUM: u32 = gl33::GL_INVALID_ENUM.0;
    #[pymodule_export]
    pub const GL_INVALID_VALUE: u32 = gl33::GL_INVALID_VALUE.0;
    #[pymodule_export]
    pub const GL_INVALID_OPERATION: u32 = gl33::GL_INVALID_OPERATION.0;
    #[pymodule_export]
    pub const GL_OUT_OF_MEMORY: u32 = gl33::GL_OUT_OF_MEMORY.0;
    #[pymodule_export]
    pub const GL_INVALID_FRAMEBUFFER_OPERATION: u32 = gl33::GL_INVALID_FRAMEBUFFER_OPERATION.0;

    #[pymodule_export]
    pub const GL_TEXTURE0: u32 = gl33::GL_TEXTURE0.0;
    #[pymodule_export]
    pub const GL_TEXTURE1: u32 = gl33::GL_TEXTURE1.0;
    #[pymodule_export]
    pub const GL_TEXTURE2: u32 = gl33::GL_TEXTURE2.0;
    #[pymodule_export]
    pub const GL_TEXTURE3: u32 = gl33::GL_TEXTURE3.0;
    #[pymodule_export]
    pub const GL_TEXTURE4: u32 = gl33::GL_TEXTURE4.0;
    #[pymodule_export]
    pub const GL_TEXTURE5: u32 = gl33::GL_TEXTURE5.0;
    #[pymodule_export]
    pub const GL_TEXTURE6: u32 = gl33::GL_TEXTURE6.0;
    #[pymodule_export]
    pub const GL_TEXTURE7: u32 = gl33::GL_TEXTURE7.0;

    #[pymodule_export]
    pub const GL_TEXTURE_BUFFER: u32 = gl33::GL_TEXTURE_BUFFER.0;
    #[pymodule_export]
    pub const GL_RGBA32F: u32 = gl33::GL_RGBA32F.0;

    #[pymodule_export]
    pub const GL_PIXEL_PACK_BUFFER: u32 = gl33::GL_PIXEL_PACK_BUFFER.0;
    #[pymodule_export]
    pub const GL_STREAM_READ: u32 = gl33::GL_STREAM_READ.0;

    #[pymodule_export]
    pub const GL_READ_ONLY: u32 = gl33::GL_READ_ONLY.0;

    #[pymodule_export]
    pub const GL_RGBA: u32 = gl33::GL_RGBA.0;
    #[pymodule_export]
    pub const GL_UNSIGNED_BYTE: u32 = gl33::GL_UNSIGNED_BYTE.0;

    #[pymodule_export]
    pub const GL_BOOL: u32 = gl33::GL_BOOL.0;
    #[pymodule_export]
    pub const GL_BOOL_VEC2: u32 = gl33::GL_BOOL_VEC2.0;
    #[pymodule_export]
    pub const GL_BOOL_VEC3: u32 = gl33::GL_BOOL_VEC3.0;
    #[pymodule_export]
    pub const GL_BOOL_VEC4: u32 = gl33::GL_BOOL_VEC4.0;

    #[pymodule_export]
    pub const GL_INT: u32 = gl33::GL_INT.0;
    #[pymodule_export]
    pub const GL_INT_VEC2: u32 = gl33::GL_INT_VEC2.0;
    #[pymodule_export]
    pub const GL_INT_VEC3: u32 = gl33::GL_INT_VEC3.0;
    #[pymodule_export]
    pub const GL_INT_VEC4: u32 = gl33::GL_INT_VEC4.0;

    #[pymodule_export]
    pub const GL_UNSIGNED_INT: u32 = gl33::GL_UNSIGNED_INT.0;
    #[pymodule_export]
    pub const GL_UNSIGNED_INT_VEC2: u32 = gl33::GL_UNSIGNED_INT_VEC2.0;
    #[pymodule_export]
    pub const GL_UNSIGNED_INT_VEC3: u32 = gl33::GL_UNSIGNED_INT_VEC3.0;
    #[pymodule_export]
    pub const GL_UNSIGNED_INT_VEC4: u32 = gl33::GL_UNSIGNED_INT_VEC4.0;

    #[pymodule_export]
    pub const GL_FLOAT: u32 = gl33::GL_FLOAT.0;
    #[pymodule_export]
    pub const GL_FLOAT_VEC2: u32 = gl33::GL_FLOAT_VEC2.0;
    #[pymodule_export]
    pub const GL_FLOAT_VEC3: u32 = gl33::GL_FLOAT_VEC3.0;
    #[pymodule_export]
    pub const GL_FLOAT_VEC4: u32 = gl33::GL_FLOAT_VEC4.0;

    #[pymodule_export]
    pub const GL_FLOAT_MAT2: u32 = gl33::GL_FLOAT_MAT2.0;
    #[pymodule_export]
    pub const GL_FLOAT_MAT3: u32 = gl33::GL_FLOAT_MAT3.0;
    #[pymodule_export]
    pub const GL_FLOAT_MAT4: u32 = gl33::GL_FLOAT_MAT4.0;

    // -----------------------------------------------------------------------
    // Native OpenGL Functions with error handling.
    // -----------------------------------------------------------------------

    #[pymodule_export]
    pub use super::functions::{is_loaded, load};

    #[pymodule_export]
    pub use super::functions::{active_texture, bind_texture, gen_textures, tex_buffer};

    #[pymodule_export]
    pub use super::functions::{
        bind_buffer, buffer_data, delete_buffers, gen_buffers, get_buffer_sub_data, map_buffer,
        unmap_buffer,
    };

    #[pymodule_export]
    pub use super::functions::{
        get_uniform_location, uniform_1f, uniform_1i, uniform_2f, uniform_bytes, use_program,
    };

    #[pymodule_export]
    pub use super::functions::read_pixels;

    // -----------------------------------------------------------------------
    // Utility Classes.
    // -----------------------------------------------------------------------

    #[pymodule_export]
    use super::uniform::FastUniform;
}
