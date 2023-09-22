use std::ffi::{c_void, CStr, CString};
use std::marker::PhantomData;

use gl::types::{GLchar, GLenum, GLfloat, GLint, GLuint, GLvoid};

pub struct TimerQuery {
    id: u32,
}

impl TimerQuery {
    pub fn new() -> TimerQuery {
        unsafe {
            let mut id = 0;
            gl::GenQueries(1, &mut id);

            TimerQuery { id }
        }
    }

    pub fn begin(&self) {
        unsafe {
            gl::BeginQuery(gl::TIME_ELAPSED, self.id);
        }
    }

    pub fn end(&self) {
        unsafe {
            gl::EndQuery(gl::TIME_ELAPSED);
        }
    }

    pub fn elapsed(&self) -> Option<u64> {
        unsafe {
            let mut available: i32 = 0;
            gl::GetQueryObjectiv(self.id, gl::QUERY_RESULT_AVAILABLE, &mut available);

            if available != 0 {
                let mut elapsed: u64 = 0;
                gl::GetQueryObjectui64v(self.id, gl::QUERY_RESULT, &mut elapsed);

                return Some(elapsed);
            }
        }

        None
    }
}

impl Drop for TimerQuery {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteQueries(1, &self.id);
        }
    }
}

pub unsafe trait UniformFormat {
    fn uniforms() -> Vec<Uniform>;
}

#[allow(unused)]
pub enum UniformType {
    Float,
    Float2,
    Float3,
    Float4,
    Float4x4,
    Texture,
}

pub struct Uniform {
    pub name: &'static CStr,
    pub type_: UniformType,
    pub offset: isize,
}

pub struct Program<U, V> {
    id: GLuint,
    marker: PhantomData<(U, V)>,
}

impl<U: UniformFormat, V: VertexFormat> Program<U, V> {
    pub fn new(vert_src: &CStr, frag_src: &CStr) -> Result<Self, String> {
        unsafe {
            let vert = shader(vert_src, gl::VERTEX_SHADER).unwrap();
            let frag = shader(frag_src, gl::FRAGMENT_SHADER).unwrap();
            let id = gl::CreateProgram();
            gl::AttachShader(id, vert);
            gl::AttachShader(id, frag);
            gl::LinkProgram(id);

            let mut valid: GLint = 1;
            gl::GetProgramiv(id, gl::LINK_STATUS, &mut valid);
            if valid == 0 {
                let mut len: GLint = 0;
                gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);
                let error = CString::new(vec![b' '; len as usize]).unwrap();
                gl::GetProgramInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);
                return Err(error.into_string().unwrap());
            }

            gl::DetachShader(id, vert);
            gl::DetachShader(id, frag);

            gl::DeleteShader(vert);
            gl::DeleteShader(frag);

            Ok(Program {
                id,
                marker: PhantomData,
            })
        }
    }

    pub fn draw(&self, uniforms: &U, mesh: &Mesh<V>) {
        unsafe {
            gl::UseProgram(self.id);

            let mut texture_slot = 0;

            for uniform in U::uniforms() {
                let location = gl::GetUniformLocation(self.id, uniform.name.as_ptr());
                let ptr = (uniforms as *const U as *const c_void).offset(uniform.offset);

                match uniform.type_ {
                    UniformType::Float => {
                        gl::Uniform1fv(location, 1, ptr as *const GLfloat);
                    }
                    UniformType::Float2 => {
                        gl::Uniform2fv(location, 1, ptr as *const GLfloat);
                    }
                    UniformType::Float3 => {
                        gl::Uniform3fv(location, 1, ptr as *const GLfloat);
                    }
                    UniformType::Float4 => {
                        gl::Uniform4fv(location, 1, ptr as *const GLfloat);
                    }
                    UniformType::Float4x4 => {
                        gl::UniformMatrix4fv(location, 1, gl::TRUE, ptr as *const GLfloat);
                    }
                    UniformType::Texture => {
                        let slot = match texture_slot {
                            0 => gl::TEXTURE0,
                            1 => gl::TEXTURE1,
                            2 => gl::TEXTURE2,
                            3 => gl::TEXTURE3,
                            4 => gl::TEXTURE4,
                            5 => gl::TEXTURE5,
                            6 => gl::TEXTURE6,
                            7 => gl::TEXTURE7,
                            _ => panic!("too many textures bound"),
                        };

                        let id = *(ptr as *const TextureId);

                        gl::ActiveTexture(slot);
                        gl::BindTexture(gl::TEXTURE_2D, id);

                        gl::Uniform1i(location, texture_slot.try_into().unwrap());

                        texture_slot += 1;
                    }
                }
            }

            gl::BindVertexArray(mesh.vao);

            gl::DrawElements(
                gl::TRIANGLES,
                mesh.len.try_into().unwrap(),
                gl::UNSIGNED_SHORT,
                0 as *const GLvoid,
            );
        }
    }
}

impl<U, V> Drop for Program<U, V> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

fn shader(shader_src: &CStr, shader_type: GLenum) -> Result<GLuint, String> {
    unsafe {
        let shader: GLuint = gl::CreateShader(shader_type);
        gl::ShaderSource(shader, 1, &shader_src.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);

        let mut valid: GLint = 1;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut valid);
        if valid == 0 {
            let mut len: GLint = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let error = CString::new(vec![b' '; len as usize]).unwrap();
            gl::GetShaderInfoLog(
                shader,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut GLchar,
            );
            return Err(error.into_string().unwrap());
        }

        Ok(shader)
    }
}

pub unsafe trait VertexFormat {
    fn attribs() -> Vec<VertexAttrib>;
}

pub enum AttribType {
    Float,
    Uint,
}

pub struct VertexAttrib {
    pub location: usize,
    pub type_: AttribType,
    pub dimension: usize,
    pub offset: isize,
}

pub struct Mesh<V> {
    vao: GLuint,
    vbo: GLuint,
    ibo: GLuint,
    len: usize,
    marker: PhantomData<V>,
}

impl<V: VertexFormat> Mesh<V> {
    pub fn new(vertices: &[V], indices: &[u16]) -> Self {
        let mut vbo: u32 = 0;
        let mut ibo: u32 = 0;
        let mut vao: u32 = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<V>()) as isize,
                vertices.as_ptr() as *const GLvoid,
                gl::DYNAMIC_DRAW,
            );

            gl::GenBuffers(1, &mut ibo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<u16>()) as isize,
                indices.as_ptr() as *const std::ffi::c_void,
                gl::DYNAMIC_DRAW,
            );

            for (index, attrib) in V::attribs().into_iter().enumerate() {
                enum Kind {
                    Float,
                    Int,
                }

                let (type_, kind) = match attrib.type_ {
                    AttribType::Float => (gl::FLOAT, Kind::Float),
                    AttribType::Uint => (gl::UNSIGNED_INT, Kind::Int),
                };

                gl::EnableVertexAttribArray(index.try_into().unwrap());
                match kind {
                    Kind::Float => {
                        gl::VertexAttribPointer(
                            attrib.location.try_into().unwrap(),
                            attrib.dimension.try_into().unwrap(),
                            type_,
                            gl::FALSE,
                            std::mem::size_of::<V>() as GLint,
                            attrib.offset as *const GLvoid,
                        );
                    }
                    Kind::Int => {
                        gl::VertexAttribIPointer(
                            attrib.location.try_into().unwrap(),
                            attrib.dimension.try_into().unwrap(),
                            type_,
                            std::mem::size_of::<V>() as GLint,
                            attrib.offset as *const GLvoid,
                        );
                    }
                }
            }
        }

        Mesh {
            vao,
            vbo,
            ibo,
            len: indices.len(),
            marker: PhantomData,
        }
    }
}

impl<V> Drop for Mesh<V> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ibo);
        }
    }
}

#[allow(unused)]
pub enum TextureFormat {
    Rg16Ui,
    Rg16Unorm,
    Rgb16Unorm,
}

pub type TextureId = GLuint;

pub struct Texture {
    id: GLuint,
}

impl Texture {
    pub unsafe fn new(
        format: TextureFormat,
        width: usize,
        height: usize,
        data: *const c_void,
    ) -> Texture {
        let mut id: GLuint = 0;
        gl::GenTextures(1, &mut id);
        gl::BindTexture(gl::TEXTURE_2D, id);

        let (internal_format, pixel_format, data_type) = match format {
            TextureFormat::Rg16Ui => (gl::RG16UI, gl::RG_INTEGER, gl::UNSIGNED_SHORT),
            TextureFormat::Rg16Unorm => (gl::RG16, gl::RG, gl::UNSIGNED_SHORT),
            TextureFormat::Rgb16Unorm => (gl::RGB16, gl::RGB, gl::UNSIGNED_SHORT),
        };

        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            internal_format as GLint,
            width.try_into().unwrap(),
            height.try_into().unwrap(),
            0,
            pixel_format,
            data_type,
            data,
        );

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

        Texture { id }
    }

    pub fn id(&self) -> TextureId {
        self.id
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
