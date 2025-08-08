mod camera;
mod graphics_context;

use crate::camera::Camera;
use bytemuck::Zeroable;
use bytemuck::{Pod, bytes_of};
use graphics_context::GraphicsContext;
use nalgebra::{Matrix4, Vector2, Vector3};
use std::borrow::Cow;
use wgpu::naga::ShaderStage;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    Buffer, BufferAddress, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, FragmentState, FrontFace, LoadOp, Operations,
    PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, PushConstantRange,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{MouseButton, WindowEvent};
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
                    .resize(new_size.to_logical(graphics_context.window.scale_factor()));
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
            _ => (),
        }
    }
}

impl App {
    pub fn render(&mut self) {
        let graphics_context = self.graphics_context.as_mut().unwrap();
        let app_context = self.app_context.as_mut().unwrap();
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
                Matrix4::<f32>::identity().append_nonuniform_scaling(&Vector3::new(32., 32., 1.));
            let mvp_matrix = view_projection_matrix * model_matrix;

            render_pass.set_vertex_buffer(0, app_context.vertex_buffer.slice(..));
            render_pass.set_pipeline(&app_context.render_pipeline);
            render_pass.set_push_constants(ShaderStages::VERTEX, 0, bytes_of(&mvp_matrix));
            render_pass.draw(0..6, 0..1);
        }
        let command_buffer = command_encoder.finish();
        graphics_context.queue.submit([command_buffer]);
        graphics_context.window.pre_present_notify();
        surface_texture.present();
        graphics_context.window.request_redraw();
    }
}

struct AppContext {
    camera: Camera,
    #[allow(unused)]
    vertex_shader: ShaderModule,
    #[allow(unused)]
    fragment_shader: ShaderModule,
    vertex_buffer: Buffer,
    render_pipeline: RenderPipeline,
}

impl AppContext {
    pub fn new(graphics_context: &GraphicsContext) -> anyhow::Result<Self> {
        // Shaders
        let vertex_shader = graphics_context
            .device
            .create_shader_module(ShaderModuleDescriptor {
                label: None,
                source: ShaderSource::Glsl {
                    shader: Cow::Borrowed(
                        r#"
                    #version 460

                    layout(location = 0) in vec3 in_Position;
                    layout(location = 1) in vec4 in_Color;
                    layout(push_constant) uniform PushConstants {
                        mat4 mvp_matrix;
                    } p_c;
                    out vec4 out_Color;

                    void main() {
                        gl_Position = p_c.mvp_matrix * vec4(in_Position, 1.0);
                        out_Color = in_Color;
                    }
                "#,
                    ),
                    stage: ShaderStage::Vertex,
                    defines: Default::default(),
                },
            });
        let fragment_shader =
            graphics_context
                .device
                .create_shader_module(ShaderModuleDescriptor {
                    label: None,
                    source: ShaderSource::Glsl {
                        shader: Cow::Borrowed(
                            r#"
                    #version 460

                    in vec4 out_Color;
                    out vec4 frag_Color;

                    void main() {
                        frag_Color = out_Color;
                    }
                "#,
                        ),
                        stage: ShaderStage::Fragment,
                        defines: Default::default(),
                    },
                });

        // Vertex buffer
        #[repr(C)]
        #[derive(Pod, Zeroable, Clone, Copy)]
        struct Vertex {
            position: [f32; 3],
            color: [f32; 3],
        }

        let vertexes = vec![
            Vertex {
                position: [-0.5, 0.5, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, 0.0],
                color: [0.0, 1.0, 0.0],
            },
        ];
        let vertex_buffer = graphics_context
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&vertexes),
                usage: BufferUsages::VERTEX,
            });

        // Render Pipeline
        let render_pipeline =
            graphics_context
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&graphics_context.device.create_pipeline_layout(
                        &PipelineLayoutDescriptor {
                            push_constant_ranges: &[PushConstantRange {
                                stages: ShaderStages::VERTEX,
                                range: 0..64,
                            }],
                            ..Default::default()
                        },
                    )),
                    vertex: VertexState {
                        module: &vertex_shader,
                        entry_point: Some("main"),
                        compilation_options: Default::default(),
                        buffers: &[VertexBufferLayout {
                            array_stride: size_of::<Vertex>() as BufferAddress,
                            step_mode: VertexStepMode::Vertex,
                            attributes: &[
                                VertexAttribute {
                                    format: VertexFormat::Float32x3,
                                    offset: 0,
                                    shader_location: 0,
                                },
                                VertexAttribute {
                                    format: VertexFormat::Float32x3,
                                    offset: 4 * 3,
                                    shader_location: 1,
                                },
                            ],
                        }],
                    },
                    primitive: PrimitiveState {
                        topology: PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: FrontFace::Cw,
                        cull_mode: None,
                        unclipped_depth: false,
                        polygon_mode: Default::default(),
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: Default::default(),
                    fragment: Some(FragmentState {
                        module: &fragment_shader,
                        entry_point: Some("main"),
                        compilation_options: Default::default(),
                        targets: &[Some(ColorTargetState {
                            format: graphics_context
                                .surface_data
                                .surface_configuration
                                .view_formats[0],
                            blend: None,
                            write_mask: ColorWrites::all(),
                        })],
                    }),
                    multiview: None,
                    cache: None,
                });

        let scale_factor = graphics_context.window.scale_factor();

        Ok(Self {
            //camera: Camera::new(graphics_context.window.inner_size().to_logical(scale_factor), scale_factor),
            camera: Camera::new(LogicalSize::new(320, 320), 1.),
            vertex_shader,
            fragment_shader,
            vertex_buffer,
            render_pipeline,
        })
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
