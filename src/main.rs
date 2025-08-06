mod graphics_context;

use graphics_context::GraphicsContext;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

#[derive(Default)]
struct App {
    graphics_context: Option<GraphicsContext>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.graphics_context.is_some() {
            return;
        }

        let graphics_context = match GraphicsContext::new(event_loop) {
            Ok(graphics_context) => graphics_context,
            Err(err) => {
                log::error!("Failed to create graphics context: {err:#}");
                event_loop.exit();
                return;
            }
        };
        self.graphics_context = Some(graphics_context);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let graphics_context = self.graphics_context.as_mut().unwrap();
        match event {
            WindowEvent::RedrawRequested => {
                graphics_context.window.request_redraw();
            }
            WindowEvent::Resized(new_size) => {
                graphics_context
                    .surface_data
                    .configure(new_size.width.max(1), new_size.height.max(1));
                graphics_context.window.request_redraw();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => (),
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().expect("Failed to create EventLoop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .expect("Failed to run app in EventLoop");
}
