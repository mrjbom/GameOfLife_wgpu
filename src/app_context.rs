use crate::camera::Camera;
use crate::graphics_context::GraphicsContext;
use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;
use wgpu::naga::ShaderStage;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    Buffer, BufferAddress, BufferUsages, ColorTargetState, ColorWrites, FragmentState, FrontFace,
    PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, PushConstantRange, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

pub struct AppContext {
    pub(crate) camera: Camera,
    #[allow(unused)]
    vertex_shader: ShaderModule,
    #[allow(unused)]
    fragment_shader: ShaderModule,
    pub(crate) vertex_buffer: Buffer,
    pub(crate) render_pipeline: RenderPipeline,
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
        let camera = Camera::new(
            graphics_context
                .window
                .inner_size()
                .to_logical(scale_factor),
            scale_factor as f32,
        );

        Ok(Self {
            camera,
            vertex_shader,
            fragment_shader,
            vertex_buffer,
            render_pipeline,
        })
    }
}
