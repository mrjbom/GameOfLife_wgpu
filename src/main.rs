mod app_context;
mod camera;
mod graphics_context;

use crate::app_context::AppContext;
use bytemuck::bytes_of;
use graphics_context::GraphicsContext;
use nalgebra::{Matrix4, Vector3};
use wgpu::{
    Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, ShaderStages, StoreOp,
};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::{MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

#[derive(Default)]
struct App {
    graphics_context: Option<GraphicsContext>,
    app_context: Option<AppContext>,
    input_state: Option<InputState>,
}

#[derive(Default, Debug)]
struct InputState {
    cursor_in_window: bool,
    lmb_is_pressed: bool,
    cursor_position: PhysicalPosition<f64>,
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

        let app_context = match AppContext::new(&graphics_context) {
            Ok(app_context) => app_context,
            Err(err) => {
                log::error!("Failed to create app context: {err:#}");
                event_loop.exit();
                return;
            }
        };

        graphics_context.window.set_visible(true);

        self.graphics_context = Some(graphics_context);
        self.app_context = Some(app_context);
        self.input_state = Some(InputState::default());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if window_id != self.graphics_context.as_ref().unwrap().window.id() {
            return;
        }
        let graphics_context = self.graphics_context.as_mut().unwrap();
        let app_context = self.app_context.as_mut().unwrap();
        let input_state = self.input_state.as_mut().unwrap();
        match event {
            WindowEvent::RedrawRequested => {
                self.render();
            }
            WindowEvent::Resized(new_size) => {
                graphics_context.surface_data.configure(new_size);
                app_context
                    .camera
                    .set_viewport_size(new_size.to_logical(graphics_context.window.scale_factor()));
                graphics_context.window.request_redraw();
            }
            WindowEvent::CloseRequested => {
                graphics_context.window.set_visible(false);
                event_loop.exit();
            }
            WindowEvent::CursorEntered { .. } => {
                input_state.cursor_in_window = true;
            }
            WindowEvent::CursorLeft { .. } => {
                input_state.cursor_in_window = false;
            }
            WindowEvent::MouseInput { button, state, .. } => {
                if button == MouseButton::Left && state.is_pressed() {
                    input_state.lmb_is_pressed = true;
                    app_context.camera.update_lmb_state(true);
                }
                if button == MouseButton::Left && !state.is_pressed() {
                    input_state.lmb_is_pressed = false;
                    app_context.camera.update_lmb_state(false);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                input_state.cursor_position = position;
                app_context.camera.update_cursor_position(
                    position.to_logical(graphics_context.window.scale_factor()),
                );
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let MouseScrollDelta::LineDelta(_, delta_y) = delta {
                    app_context.camera.mouse_scroll(delta_y);
                } else {
                    unimplemented!("MouseScrollDelta::PixelDelta event unimplemented!");
                }
            }
            _ => (),
        }
    }
}

impl App {
    pub fn render(&mut self) {
        let graphics_context = self.graphics_context.as_mut().unwrap();
        let app_context = self.app_context.as_mut().unwrap();
        let input_state = self.input_state.as_ref().unwrap();
        let (surface_texture, surface_texture_view) = graphics_context.surface_data.acquire();

        let mut command_encoder = graphics_context
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &surface_texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            let view_projection_matrix = app_context.camera.calculate_view_projection_matrix();
            let model_matrix =
                Matrix4::<f32>::identity().append_nonuniform_scaling(&Vector3::new(128., 128., 1.));
            let mvp_matrix = view_projection_matrix * model_matrix;

            render_pass.set_vertex_buffer(0, app_context.vertex_buffer.slice(..));
            render_pass.set_pipeline(&app_context.render_pipeline);
            render_pass.set_push_constants(ShaderStages::VERTEX, 0, bytes_of(&mvp_matrix));
            render_pass.draw(0..6, 0..1);

            // Square under cursor
            /*
            let cursor_position = input_state.cursor_position.to_logical(graphics_context.window.scale_factor());
            let cursor_world_position = app_context.camera.screen_to_world_position(cursor_position);
            let scale = Matrix4::<f32>::identity().append_nonuniform_scaling(&Vector3::new(32., 32., 1.));
            let translation = Matrix4::<f32>::identity().append_translation(&cursor_world_position.push(0.));
            let model_matrix = translation * scale;
            let mvp_matrix = view_projection_matrix * model_matrix;
            render_pass.set_push_constants(ShaderStages::VERTEX, 0, bytes_of(&mvp_matrix));
            render_pass.draw(0..6, 0..1);
             */
        }
        let command_buffer = command_encoder.finish();
        graphics_context.queue.submit([command_buffer]);
        graphics_context.window.pre_present_notify();
        surface_texture.present();
        graphics_context.window.request_redraw();
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
