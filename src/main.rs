use anyhow::Context;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

#[derive(Default)]
struct App {
    graphics_context: Option<GraphicsContext>,
}

struct GraphicsContext {
    window: Window,
}

impl GraphicsContext {
    fn new(event_loop: &ActiveEventLoop) -> anyhow::Result<Self> {
        let primary_monitor = event_loop
            .primary_monitor()
            .context("Failed to get primary monitor")?;

        let monitor_size = primary_monitor.size();
        let window_size = PhysicalSize::new(
            (monitor_size.width as f32 * 0.75) as u32,
            (monitor_size.height as f32 * 0.75) as u32,
        );
        let window_position = {
            let x = (monitor_size.width / 2 - window_size.width / 2) as i32;
            let y = (monitor_size.height / 2 - window_size.height / 2) as i32;
            PhysicalPosition::new(x, y)
        };

        let window_attributes = WindowAttributes::default()
            .with_inner_size(window_size)
            .with_position(window_position);

        let window = event_loop
            .create_window(window_attributes)
            .context("Failed to create window")?;
        Ok(GraphicsContext { window })
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
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                graphics_context.window.request_redraw();
            }
            _ => (),
        }
    }
}
