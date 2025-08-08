mod surface_data;

use crate::graphics_context::surface_data::SurfaceData;
use anyhow::Context;
use std::sync::Arc;
use wgpu::{
    Adapter, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits,
    PowerPreference, Queue, RequestAdapterOptions,
};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

pub struct GraphicsContext {
    pub window: Arc<Window>,
    #[allow(unused)]
    pub instance: Instance,
    #[allow(unused)]
    pub adapter: Adapter,
    pub device: Arc<Device>,
    pub queue: Queue,
    pub surface_data: SurfaceData,
}

impl GraphicsContext {
    pub fn new(event_loop: &ActiveEventLoop) -> anyhow::Result<Self> {
        let window = Arc::new(create_window(&event_loop)?);

        let instance = Instance::new(&InstanceDescriptor::default());
        let surface = instance
            .create_surface(Arc::clone(&window))
            .context("Failed to create surface")?;
        let adapter =
            futures::executor::block_on(instance.request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            }))
            .context("Failed to request adapter")?;
        let (device, queue) =
            futures::executor::block_on(adapter.request_device(&DeviceDescriptor {
                required_features: Features::default() | Features::PUSH_CONSTANTS,
                required_limits: Limits {
                    // 4x4 matrix
                    max_push_constant_size: 64,
                    ..Default::default()
                },
                ..Default::default()
            }))
            .context("Failed to request device and queue")?;
        let device = Arc::new(device);
        let mut surface_data =
            SurfaceData::new(Arc::clone(&window), surface, &adapter, Arc::clone(&device));
        surface_data.configure(window.inner_size());
        window.request_redraw();

        Ok(GraphicsContext {
            window,
            instance,
            adapter,
            device,
            queue,
            surface_data,
        })
    }
}

fn create_window(event_loop: &ActiveEventLoop) -> anyhow::Result<Window> {
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
        .with_min_inner_size(PhysicalSize::new(1, 1))
        .with_inner_size(window_size)
        .with_position(window_position)
        .with_visible(false);

    event_loop
        .create_window(window_attributes)
        .context("Failed to create window")
}
