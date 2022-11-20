use std::ffi::{CStr, CString};

use gl::types::{GLchar, GLenum, GLint, GLuint, GLvoid};

pub struct TimerQuery {
    query: u32,
}

impl TimerQuery {
    pub fn new() -> TimerQuery {
        unsafe {
            let mut query = 0;
            gl::GenQueries(1, &mut query);

            TimerQuery { query: query }
        }
    }

    pub fn begin(&self) {
        unsafe {
            gl::BeginQuery(gl::TIME_ELAPSED, self.query);
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
            gl::GetQueryObjectiv(self.query, gl::QUERY_RESULT_AVAILABLE, &mut available);

            if available != 0 {
                let mut elapsed: u64 = 0;
                gl::GetQueryObjectui64v(self.query, gl::QUERY_RESULT, &mut elapsed);

                return Some(elapsed);
            }
        }

        None
    }
}

impl Drop for TimerQuery {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteQueries(1, &self.query);
        }
    }
}

pub struct Program {
    id: GLuint,
}

impl Program {
    pub fn new(vert_src: &CStr, frag_src: &CStr) -> Result<Program, String> {
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

            Ok(Program { id })
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
}

impl Drop for Program {
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
