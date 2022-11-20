mod geom;
mod path;
mod render;
mod window;

use std::collections::{HashMap, VecDeque};
use std::ffi::CString;

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
    transform: [f32; 16],
}

unsafe impl UniformFormat for GlyphUniforms {
    fn uniforms() -> Vec<Uniform> {
        vec![Uniform {
            location: 0,
            type_: UniformType::Float2,
            offset: unsafe { offset_of!(GlyphUniforms, screen_size) },
        }, Uniform {
            location: 1,
            type_: UniformType::Float4x4,
            offset: unsafe { offset_of!(GlyphUniforms, transform) },
        }]
    }
}

type GlyphId = u16;

struct GlyphEntry {
    path: Path,
    components_range: [u32; 2],
    points_range: [u32; 2],
}

struct Text {
    font: Font<'static>,
    glyph_cache: HashMap<GlyphId, GlyphEntry>,
    components: Vec<u16>,
    points: Vec<u16>,
    texture_width: usize,
    components_len: usize,
    points_len: usize,
}

impl Text {
    fn with_texture_width(texture_width: usize) -> Text {
        let font = Font::from_data(include_bytes!("SourceSansPro-Regular.otf"), 0).unwrap();

        Text {
            font,
            glyph_cache: HashMap::new(),
            components: Vec::new(),
            points: Vec::new(),
            texture_width,
            components_len: 0,
            points_len: 0,
        }
    }

    fn layout(&mut self, size: f32, text: &str) -> TextLayout {
        let scale = size / self.font.units_per_em().unwrap() as f32;
        let line_height = scale * (self.font.height() + self.font.line_gap()) as f32;

        let mut glyphs = Vec::new();
        let mut width: f32 = 0.0;
        let mut height: f32 = line_height;

        let mut pos = Vec2::new(0.0, scale * self.font.ascender() as f32);

        for c in text.chars() {
            if c == '\n' {
                pos.x = 0.0;
                pos.y -= line_height;
                height += line_height;
            } else if let Ok(glyph_id) = self.font.glyph_index(c) {
                glyphs.push(Glyph {
                    id: glyph_id.0,
                    pos,
                });

                pos.x += scale * self.font.glyph_hor_metrics(glyph_id).unwrap().advance as f32;
                width = width.max(pos.x);
            }
        }

        TextLayout {
            glyphs,
            scale,
            width,
            height,
        }
    }

    fn mesh(&mut self, layout: &TextLayout) -> Mesh<GlyphVertex> {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for glyph in layout.glyphs.iter() {
            let glyph_entry = self.glyph_cache.entry(glyph.id).or_insert_with(|| {
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
                let _ = self.font.outline_glyph(ttf_parser::GlyphId(glyph.id), &mut builder);

                let path = builder.path.build();

                if self.components_len + path.components.len() > self.components.len() {
                    self.components
                        .resize(self.components.len() + self.texture_width, 0);
                }
                self.components[self.components_len..self.components_len + path.components.len()]
                    .clone_from_slice(&path.components);
                let components_start = self.components_len / 2;
                let components_end = (self.components_len + path.components.len()) / 2;
                let components_range = [
                    components_start.try_into().unwrap(),
                    components_end.try_into().unwrap(),
                ];
                self.components_len += path.components.len();

                if self.points_len + path.points.len() > self.points.len() {
                    self.points
                        .resize(self.points.len() + self.texture_width, 0);
                }
                self.points[self.points_len..self.points_len + path.points.len()]
                    .clone_from_slice(&path.points);
                let points_start = self.points_len / 2;
                let points_end = (self.points_len + path.points.len()) / 2;
                let points_range = [
                    points_start.try_into().unwrap(),
                    points_end.try_into().unwrap(),
                ];
                self.points_len += path.points.len();

                GlyphEntry {
                    path,
                    components_range,
                    points_range,
                }
            });

            let base: u16 = vertices.len().try_into().unwrap();
            vertices.extend_from_slice(&[
                GlyphVertex {
                    pos: [
                        glyph.pos.x + layout.scale * glyph_entry.path.min.x,
                        glyph.pos.y + layout.scale * glyph_entry.path.min.y,
                        0.0,
                    ],
                },
                GlyphVertex {
                    pos: [
                        glyph.pos.x + layout.scale * glyph_entry.path.max.x,
                        glyph.pos.y + layout.scale * glyph_entry.path.min.y,
                        0.0,
                    ],
                },
                GlyphVertex {
                    pos: [
                        glyph.pos.x + layout.scale * glyph_entry.path.max.x,
                        glyph.pos.y + layout.scale * glyph_entry.path.max.y,
                        0.0,
                    ],
                },
                GlyphVertex {
                    pos: [
                        glyph.pos.x + layout.scale * glyph_entry.path.min.x,
                        glyph.pos.y + layout.scale * glyph_entry.path.max.y,
                        0.0,
                    ],
                },
            ]);
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }

        Mesh::new(&vertices, &indices)
    }
}

struct Glyph {
    id: GlyphId,
    pos: Vec2,
}

struct TextLayout {
    glyphs: Vec<Glyph>,
    scale: f32,
    width: f32,
    height: f32,
}

impl TextLayout {
    fn width(&self) -> f32 {
        self.width
    }

    fn height(&self) -> f32 {
        self.height
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

const TEXTURE_WIDTH: usize = 4096;

struct GouacheHandler {
    timers: VecDeque<TimerQuery>,
    prog: Program<GlyphUniforms, GlyphVertex>,
    mesh: Mesh<GlyphVertex>,

    layout: TextLayout,

    dragging: bool,
    cursor: Vec2,
    z: f32,
    rotate: Mat4x4,
}

impl Handler for GouacheHandler {
    fn scroll(&mut self, _dx: f32, dy: f32) {
        self.z *= 0.995f32.powf(-dy);
    }

    fn mouse_down(&mut self) {
        self.dragging = true;
    }

    fn mouse_up(&mut self) {
        self.dragging = false;
    }

    fn mouse_move(&mut self, x: f32, y: f32) {
        let new_cursor = Vec2::new(x, y);

        if self.dragging {
            let delta = new_cursor - self.cursor;
            self.rotate *= Mat4x4::rotate_zx(delta.x * std::f32::consts::PI / 512.0);
            self.rotate *= Mat4x4::rotate_yz(delta.y * std::f32::consts::PI / 512.0);
        }

        self.cursor = new_cursor;
    }

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

        let model = Mat4x4::scale(0.1)
            * Mat4x4::translate(-0.5 * self.layout.width(), 0.5 * self.layout.height(), 0.0);
        let view = Mat4x4::translate(0.0, 0.0, -1.0 - self.z) * self.rotate;
        let proj = Mat4x4::perspective(std::f32::consts::PI / 4.0, SCREEN_WIDTH / SCREEN_HEIGHT, 0.1, 10000.0);
        let transform = proj * view * model;

        let timer = TimerQuery::new();
        timer.begin();

        self.prog.draw(
            &GlyphUniforms {
                screen_size: [SCREEN_WIDTH, SCREEN_HEIGHT],
                transform: transform.0,
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

    let timers: VecDeque<TimerQuery> = VecDeque::with_capacity(64);

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

    let mut text = Text::with_texture_width(TEXTURE_WIDTH);

    let layout = text.layout(SIZE, TEXT);
    let mesh = text.mesh(&layout);

    window.run(GouacheHandler {
        timers,
        prog,
        mesh,

        layout,

        dragging: false,
        cursor: Vec2::new(-1.0, -1.0),
        z: 70.0,
        rotate: Mat4x4::id(),
    });
}
