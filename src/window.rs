use glutin::dpi::LogicalSize;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{Api, ContextBuilder, ContextWrapper, GlRequest, PossiblyCurrent};

pub type GlContext = ContextWrapper<PossiblyCurrent, glutin::window::Window>;

pub trait Handler {
    fn scroll(&mut self, dx: f32, dy: f32) {
        let _ = (dx, dy);
    }
    fn mouse_down(&mut self) {}
    fn mouse_up(&mut self) {}
    fn mouse_move(&mut self, x: f32, y: f32) {
        let _ = (x, y);
    }
    fn render(&mut self, context: &GlContext) {
        let _ = context;
    }
}

pub struct Window {
    event_loop: EventLoop<()>,
    context: ContextWrapper<PossiblyCurrent, glutin::window::Window>,
}

impl Window {
    pub fn open(width: f32, height: f32) -> Window {
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

        Window {
            event_loop,
            context,
        }
    }

    pub fn run(self, mut handler: impl Handler + 'static)
    where
        Self: 'static,
    {
        self.event_loop
            .run(move |event, _, control_flow| match event {
                Event::MainEventsCleared => {
                    self.context.window().request_redraw();
                }
                Event::RedrawRequested(..) => {
                    handler.render(&self.context);
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::MouseWheel { delta, .. } => match delta {
                        glutin::event::MouseScrollDelta::PixelDelta(position) => {
                            handler.scroll(position.x as f32, position.y as f32);
                        }
                        glutin::event::MouseScrollDelta::LineDelta(dx, dy) => {
                            handler.scroll(dx as f32 * 12.0, dy as f32 * 12.0);
                        }
                    },
                    WindowEvent::MouseInput {
                        button: glutin::event::MouseButton::Left,
                        state,
                        ..
                    } => match state {
                        glutin::event::ElementState::Pressed => {
                            handler.mouse_down();
                        }
                        glutin::event::ElementState::Released => {
                            handler.mouse_up();
                        }
                    },
                    WindowEvent::CursorMoved { position, .. } => {
                        handler.mouse_move(position.x as f32, position.y as f32);
                    }
                    _ => {}
                },
                _ => {}
            });
    }
}
