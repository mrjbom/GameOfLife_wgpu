use crate::camera::Camera;
use crate::graphics_context::GraphicsContext;
use bytemuck::{Pod, Zeroable};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    Buffer, BufferAddress, BufferUsages, ColorTargetState, ColorWrites, FragmentState, FrontFace,
    PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, PushConstantRange, RenderPipeline,
    RenderPipelineDescriptor, ShaderStages, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexState, VertexStepMode, include_wgsl,
};

pub struct AppContext {
    pub camera: Camera,
    pub vertex_buffer: Buffer,
    pub render_pipeline: RenderPipeline,
}

impl AppContext {
    pub fn new(graphics_context: &GraphicsContext) -> anyhow::Result<Self> {
        // Shaders
        let shader_module = graphics_context
            .device
            .create_shader_module(include_wgsl!("shaders/vs_fs.wgsl"));

        // Vertex buffer
        #[repr(C)]
        #[derive(Pod, Zeroable, Clone, Copy)]
        struct Vertex {
            position: [f32; 2],
            color: [f32; 3],
        }

        let vertexes = vec![
            Vertex {
                position: [-0.5, 0.5],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5],
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
                        module: &shader_module,
                        entry_point: Some("vs_main"),
                        compilation_options: Default::default(),
                        buffers: &[VertexBufferLayout {
                            array_stride: size_of::<Vertex>() as BufferAddress,
                            step_mode: VertexStepMode::Vertex,
                            attributes: &[
                                VertexAttribute {
                                    format: VertexFormat::Float32x2,
                                    offset: 0,
                                    shader_location: 0,
                                },
                                VertexAttribute {
                                    format: VertexFormat::Float32x3,
                                    offset: 4 * 2,
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
                        module: &shader_module,
                        entry_point: Some("fs_main"),
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
        );

        Ok(Self {
            camera,
            vertex_buffer,
            render_pipeline,
        })
    }
}
