mod geom;

use std::collections::{HashMap, VecDeque};
use std::ffi::{CStr, CString};

use gl::types::{GLchar, GLenum, GLint, GLuint, GLvoid};

use glutin::dpi::LogicalSize;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::{Window, WindowBuilder};
use glutin::{Api, ContextBuilder, ContextWrapper, GlRequest, PossiblyCurrent};

use ttf_parser::Font;

use geom::*;

macro_rules! offset_of {
    ($struct:ty, $field:ident) => {{
        use ::std::ffi::c_void;

        let dummy = ::std::mem::MaybeUninit::<$struct>::uninit();
        let base = dummy.as_ptr();
        let field = ::std::ptr::addr_of!((*base).$field);

        (field as *const c_void).offset_from(base as *const c_void)
    }};
}

struct TimerQuery {
    query: u32,
}

impl TimerQuery {
    fn new() -> TimerQuery {
        unsafe {
            let mut query = 0;
            gl::GenQueries(1, &mut query);

            TimerQuery { query: query }
        }
    }

    fn begin(&self) {
        unsafe {
            gl::BeginQuery(gl::TIME_ELAPSED, self.query);
        }
    }

    fn end(&self) {
        unsafe {
            gl::EndQuery(gl::TIME_ELAPSED);
        }
    }

    fn elapsed(&self) -> Option<u64> {
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

struct Program {
    id: GLuint,
}

impl Program {
    fn new(vert_src: &CStr, frag_src: &CStr) -> Result<Program, String> {
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

    fn bind(&self) {
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

struct GouacheWindow {
    event_loop: EventLoop<()>,
    context: ContextWrapper<PossiblyCurrent, Window>,
}

impl GouacheWindow {
    fn open(width: f32, height: f32) -> GouacheWindow {
        let event_loop = EventLoop::new();
        let window_builder = WindowBuilder::new()
            .with_inner_size(LogicalSize::new(width, height))
            .with_title("gouache");
        let context = ContextBuilder::new()
            .with_gl(GlRequest::Specific(Api::OpenGl, (4, 6)))
            .with_vsync(false)
            .build_windowed(window_builder, &event_loop)
            .unwrap();
        let context = unsafe { context.make_current() }.unwrap();

        gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

        GouacheWindow {
            event_loop,
            context,
        }
    }

    fn run(self, mut draw: impl FnMut(&ContextWrapper<PossiblyCurrent, Window>) + 'static)
    where
        Self: 'static,
    {
        self.event_loop
            .run(move |event, _, control_flow| match event {
                Event::MainEventsCleared => {
                    self.context.window().request_redraw();
                }
                Event::RedrawRequested(..) => {
                    draw(&self.context);
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                },
                _ => {}
            });
    }
}

struct Component {
    start: usize,
    end: usize,
}

struct PathBuilder {
    components: Vec<Component>,
    points: Vec<Vec2>,
}

impl PathBuilder {
    fn new() -> PathBuilder {
        PathBuilder {
            components: Vec::new(),
            points: Vec::new(),
        }
    }

    fn add_point(&mut self, point: Vec2) {
        if let Some(component) = self.components.last_mut() {
            component.end += 1;
        }
        self.points.push(point);
    }

    pub fn move_to(&mut self, point: Vec2) -> &mut Self {
        self.components.push(Component {
            start: self.points.len(),
            end: self.points.len(),
        });
        self.add_point(point);
        self
    }

    pub fn line_to(&mut self, point: Vec2) -> &mut Self {
        self.add_point(point);
        self.add_point(point);
        self
    }

    pub fn quadratic_to(&mut self, control: Vec2, point: Vec2) -> &mut Self {
        self.add_point(control);
        self.add_point(point);
        self
    }

    pub fn cubic_to(&mut self, control1: Vec2, control2: Vec2, point: Vec2) -> &mut Self {
        let last = self.points.last().cloned().unwrap_or(Vec2::new(0.0, 0.0));

        let width = last.x.max(control1.x).max(control2.x).max(point.x)
            - last.x.min(control1.x).min(control2.x).min(point.x);
        let height = last.y.max(control1.y).max(control2.y).max(point.y)
            - last.y.min(control1.y).min(control2.y).min(point.y);
        let factor = 0.001 * width.max(height) * 18.0 / 3.0f32.sqrt();

        let mut p1 = self.points.last().cloned().unwrap_or(Vec2::new(0.0, 0.0));
        let mut p2 = control1;
        let mut p3 = control2;
        let p4 = point;
        loop {
            let error = (3.0 * p2 - 3.0 * p3 - p1 + p4).length();
            let split = (factor / error).cbrt();

            if error == 0.0 || split > 1.0 {
                break;
            }

            let p12 = Vec2::lerp(split, p1, p2);
            let p23 = Vec2::lerp(split, p2, p3);
            let p34 = Vec2::lerp(split, p3, p4);
            let p123 = Vec2::lerp(split, p12, p23);
            let p234 = Vec2::lerp(split, p23, p34);
            let p = Vec2::lerp(split, p123, p234);

            self.quadratic_to(0.25 * (3.0 * p12 + 3.0 * p123 - p1 - p), p);

            p1 = p;
            p2 = p234;
            p3 = p34;
        }

        self.quadratic_to(0.25 * (3.0 * p2 + 3.0 * p3 - p1 - p4), p4);

        self
    }

    pub fn close(&mut self) {
        if let Some(component) = self.components.last_mut() {
            let first = self.points[component.start];
            let last = self.points[component.end - 1];
            if first != last {
                self.add_point(first);
                self.add_point(first);
            }
        }
    }

    fn build(&self) -> Path {
        if self.points.is_empty() {
            return Path {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(0.0, 0.0),
            };
        }

        let mut min = self.points[0];
        let mut max = self.points[0];

        for &point in self.points.iter() {
            min = min.min(point);
            max = max.max(point);
        }

        Path { min, max }
    }
}

struct Path {
    min: Vec2,
    max: Vec2,
}

#[derive(Copy, Clone)]
#[repr(C)]
struct Vertex {
    pos: [f32; 3],
}

struct Text {
    font: Font<'static>,
    glyph_cache: HashMap<u16, Path>,
}

impl Text {
    fn new() -> Text {
        let font = Font::from_data(include_bytes!("SourceSansPro-Regular.otf"), 0).unwrap();

        Text {
            font,
            glyph_cache: HashMap::new(),
        }
    }

    fn layout(&mut self, offset: Vec2, size: f32, text: &str) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let scale = size / self.font.units_per_em().unwrap() as f32;

        let mut pos = offset + Vec2::new(0.0, scale * self.font.ascender() as f32);

        let line_height = scale * (self.font.height() + self.font.line_gap()) as f32;
        let mut width: f32 = 0.0;
        let mut height: f32 = line_height;
        for c in text.chars() {
            if c == '\n' {
                pos.x = 0.0;
                pos.y -= line_height;
                height += line_height;
            } else if let Ok(glyph_id) = self.font.glyph_index(c) {
                let path = self.glyph_cache.entry(glyph_id.0).or_insert_with(|| {
                    use ttf_parser::OutlineBuilder;

                    struct Builder {
                        path: PathBuilder,
                    }
                    impl OutlineBuilder for Builder {
                        fn move_to(&mut self, x: f32, y: f32) {
                            self.path.move_to(Vec2::new(x, y));
                        }
                        fn line_to(&mut self, x: f32, y: f32) {
                            self.path.line_to(Vec2::new(x, y));
                        }
                        fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
                            self.path.quadratic_to(Vec2::new(x1, y1), Vec2::new(x, y));
                        }
                        fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
                            self.path.cubic_to(
                                Vec2::new(x1, y1),
                                Vec2::new(x2, y2),
                                Vec2::new(x, y),
                            );
                        }
                        fn close(&mut self) {
                            self.path.close();
                        }
                    }

                    let mut builder = Builder {
                        path: PathBuilder::new(),
                    };
                    let _ = self.font.outline_glyph(glyph_id, &mut builder);

                    builder.path.build()
                });

                let base: u16 = vertices.len().try_into().unwrap();
                vertices.extend_from_slice(&[
                    Vertex {
                        pos: [pos.x + scale * path.min.x, pos.y + scale * path.min.y, 0.0],
                    },
                    Vertex {
                        pos: [pos.x + scale * path.max.x, pos.y + scale * path.min.y, 0.0],
                    },
                    Vertex {
                        pos: [pos.x + scale * path.max.x, pos.y + scale * path.max.y, 0.0],
                    },
                    Vertex {
                        pos: [pos.x + scale * path.min.x, pos.y + scale * path.max.y, 0.0],
                    },
                ]);
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

                pos.x += scale * self.font.glyph_hor_metrics(glyph_id).unwrap().advance as f32;
                width = width.max(pos.x);
            }
        }

        (vertices, indices)
    }
}

fn main() {
    const SCREEN_WIDTH: f32 = 800.0;
    const SCREEN_HEIGHT: f32 = 600.0;
    let window = GouacheWindow::open(SCREEN_WIDTH, SCREEN_HEIGHT);

    let mut timers: VecDeque<TimerQuery> = VecDeque::with_capacity(64);

    let prog = Program::new(
        &CString::new(include_bytes!("vert.glsl") as &[u8]).unwrap(),
        &CString::new(include_bytes!("frag.glsl") as &[u8]).unwrap(),
    )
    .unwrap();

    unsafe {
        gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::BLEND);
        gl::Enable(gl::FRAMEBUFFER_SRGB);
    }

    const TEXT: &'static str =
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor
incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis
nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu
fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in
culpa qui officia deserunt mollit anim id est laborum.

Curabitur pretium tincidunt lacus. Nulla gravida orci a odio. Nullam varius,
turpis et commodo pharetra, est eros bibendum elit, nec luctus magna felis
sollicitudin mauris. Integer in mauris eu nibh euismod gravida. Duis ac tellus
et risus vulputate vehicula. Donec lobortis risus a elit. Etiam tempor. Ut
ullamcorper, ligula eu tempor congue, eros est euismod turpis, id tincidunt
sapien risus a quam. Maecenas fermentum consequat mi. Donec fermentum.
Pellentesque malesuada nulla a mi. Duis sapien sem, aliquet nec, commodo eget,
consequat quis, neque. Aliquam faucibus, elit ut dictum aliquet, felis nisl
adipiscing sapien, sed malesuada diam lacus eget erat. Cras mollis scelerisque
nunc. Nullam arcu. Aliquam consequat. Curabitur augue lorem, dapibus quis,
laoreet et, pretium ac, nisi. Aenean magna nisl, mollis quis, molestie eu,
feugiat in, orci. In hac habitasse platea dictumst.";

    const SIZE: f32 = 18.0;

    let mut text = Text::new();
    let (vertices, indices) = text.layout(Vec2::new(0.0, -2.0 * SIZE + SCREEN_HEIGHT), SIZE, TEXT);

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
            (vertices.len() * std::mem::size_of::<Vertex>()) as isize,
            vertices.as_ptr() as *const std::ffi::c_void,
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

        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            std::mem::size_of::<Vertex>() as GLint,
            offset_of!(Vertex, pos) as *const GLvoid,
        );
    }

    window.run(move |context| {
        while let Some(timer) = timers.front() {
            if let Some(elapsed) = timer.elapsed() {
                println!("elapsed: {}", elapsed);
                let _ = timers.pop_front();
            } else {
                break;
            }
        }

        unsafe {
            gl::ClearColor(0.7, 0.7, 0.75, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        prog.bind();

        unsafe {
            gl::Uniform2f(0, SCREEN_WIDTH, SCREEN_HEIGHT);
        }

        unsafe {
            gl::BindVertexArray(vao);
        }

        let timer = TimerQuery::new();
        timer.begin();

        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                vertices.len() as i32,
                gl::UNSIGNED_SHORT,
                0 as *const GLvoid,
            );
        }

        timer.end();
        timers.push_back(timer);

        context.swap_buffers().unwrap();
    });
}
