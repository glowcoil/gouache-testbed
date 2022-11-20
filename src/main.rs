mod geom;
mod path;
mod render;
mod window;

use std::collections::{HashMap, VecDeque};
use std::ffi::{CStr, CString};

use gl::types::{GLchar, GLenum, GLint, GLuint, GLvoid};
use ttf_parser::Font;

use geom::*;
use path::*;
use render::*;
use window::*;

macro_rules! offset_of {
    ($struct:ty, $field:ident) => {{
        use ::std::ffi::c_void;

        let dummy = ::std::mem::MaybeUninit::<$struct>::uninit();
        let base = dummy.as_ptr();
        let field = ::std::ptr::addr_of!((*base).$field);

        (field as *const c_void).offset_from(base as *const c_void)
    }};
}

#[derive(Copy, Clone)]
#[repr(C)]
struct GlyphVertex {
    pos: [f32; 3],
}

unsafe impl VertexFormat for GlyphVertex {
    fn attribs() -> Vec<VertexAttrib> {
        vec![VertexAttrib {
            location: 0,
            type_: AttribType::Float,
            dimension: 3,
            offset: unsafe { offset_of!(GlyphVertex, pos) },
        }]
    }
}

#[repr(C)]
struct GlyphUniforms {
    screen_size: [f32; 2],
}

unsafe impl UniformFormat for GlyphUniforms {
    fn uniforms() -> Vec<Uniform> {
        vec![Uniform {
            location: 0,
            type_: UniformType::Float2,
            offset: unsafe { offset_of!(GlyphUniforms, screen_size) },
        }]
    }
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

    fn layout(&mut self, offset: Vec2, size: f32, text: &str) -> Mesh<GlyphVertex> {
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
                    GlyphVertex {
                        pos: [pos.x + scale * path.min.x, pos.y + scale * path.min.y, 0.0],
                    },
                    GlyphVertex {
                        pos: [pos.x + scale * path.max.x, pos.y + scale * path.min.y, 0.0],
                    },
                    GlyphVertex {
                        pos: [pos.x + scale * path.max.x, pos.y + scale * path.max.y, 0.0],
                    },
                    GlyphVertex {
                        pos: [pos.x + scale * path.min.x, pos.y + scale * path.max.y, 0.0],
                    },
                ]);
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

                pos.x += scale * self.font.glyph_hor_metrics(glyph_id).unwrap().advance as f32;
                width = width.max(pos.x);
            }
        }

        Mesh::new(&vertices, &indices)
    }
}

const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;

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

struct GouacheHandler {
    timers: VecDeque<TimerQuery>,
    prog: Program<GlyphUniforms, GlyphVertex>,
    mesh: Mesh<GlyphVertex>,
}

impl Handler for GouacheHandler {
    fn scroll(&mut self, dx: f32, dy: f32) {}
    fn mouse_down(&mut self) {}
    fn mouse_up(&mut self) {}
    fn mouse_move(&mut self, dx: f32, dy: f32) {}

    fn render(&mut self, context: &GlContext) {
        while let Some(timer) = self.timers.front() {
            if let Some(elapsed) = timer.elapsed() {
                println!("elapsed: {}", elapsed);
                let _ = self.timers.pop_front();
            } else {
                break;
            }
        }

        unsafe {
            gl::ClearColor(0.7, 0.7, 0.75, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        let timer = TimerQuery::new();
        timer.begin();

        self.prog.draw(
            &GlyphUniforms {
                screen_size: [SCREEN_WIDTH, SCREEN_HEIGHT],
            },
            &self.mesh,
        );

        timer.end();
        self.timers.push_back(timer);

        context.swap_buffers().unwrap();
    }
}

fn main() {
    let window = Window::open(SCREEN_WIDTH, SCREEN_HEIGHT);

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

    let mut text = Text::new();
    let mesh = text.layout(Vec2::new(0.0, -2.0 * SIZE + SCREEN_HEIGHT), SIZE, TEXT);

    window.run(GouacheHandler { timers, prog, mesh });
}
