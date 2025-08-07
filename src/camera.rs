// 2D Camera

use nalgebra::{Matrix4, Vector2, Vector3};

#[derive(Debug)]
pub struct Camera {
    position: Vector2<f32>,
    zoom: f32,
    viewport_size: Vector2<u32>,
}

impl Camera {
    pub fn new(viewport_width: u32, viewport_height: u32) -> Self {
        Self {
            position: Vector2::new(0., 0.),
            zoom: 1.,
            viewport_size: Vector2::new(viewport_width, viewport_height),
        }
    }

    pub fn position(&self) -> Vector2<f32> {
        self.position
    }

    pub fn set_position(&mut self, position: Vector2<f32>) {
        self.position = position
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom
    }

    pub fn viewport_size(&self) -> Vector2<u32> {
        self.viewport_size
    }

    pub fn set_viewport_size(&mut self, viewport_size: Vector2<u32>) {
        self.viewport_size = viewport_size
    }

    pub fn calculate_view_projection_matrix(&self) -> Matrix4<f32> {
        let half_w = (self.viewport_size.x as f32) / (2.0 * self.zoom);
        let half_h = (self.viewport_size.y as f32) / (2.0 * self.zoom);

        let left = -half_w;
        let right = half_w;
        let bottom = -half_h;
        let top = half_h;

        let proj = Matrix4::new_orthographic(left, right, bottom, top, -1.0, 1.0);
        let view = Matrix4::new_translation(&Vector3::new(-self.position.x, -self.position.y, 0.0));

        proj * view
    }

    fn viewport_to_world(&self, screen_pos: Vector2<f32>) -> Vector2<f32> {
        let screen_center = Vector2::new(
            self.viewport_size.x as f32 / 2.0,
            self.viewport_size.y as f32 / 2.0,
        );

        let screen_offset = screen_pos.cast() - screen_center;
        let world_offset = screen_offset / self.zoom;

        self.position + world_offset
    }

    pub fn resize(&mut self, viewport_width: u32, viewport_height: u32) {
        self.viewport_size.x = viewport_width;
        self.viewport_size.y = viewport_height;
    }

    pub fn process_mouse_motion(&mut self, delta_x: f32, delta_y: f32) {
        self.position.x += -delta_x;
        self.position.y += delta_y;
    }
}
