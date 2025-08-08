// 2D Camera

use float_cmp::approx_eq;
use nalgebra::{Matrix4, Vector2, Vector3};
use winit::dpi::{LogicalPosition, LogicalSize};

#[derive(Debug)]
pub struct Camera {
    position: Vector2<f32>,
    zoom: f32,
    viewport_size: LogicalSize<u32>,
    scale_factor: f32,
    lmb_is_pressed: bool,
    cursor_position: LogicalPosition<f32>,
}

impl Camera {
    pub fn new(viewport_size: LogicalSize<u32>, scale_factor: f32) -> Self {
        Self {
            position: Vector2::new(0., 0.),
            zoom: 1.,
            viewport_size,
            scale_factor,
            lmb_is_pressed: false,
            cursor_position: LogicalPosition::new(0., 0.),
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

    pub fn viewport_size(&self) -> LogicalSize<u32> {
        self.viewport_size
    }

    pub fn set_viewport_size(&mut self, viewport_size: LogicalSize<u32>) {
        self.viewport_size = viewport_size
    }

    pub fn calculate_view_projection_matrix(&self) -> Matrix4<f32> {
        let half_w = (self.viewport_size.width as f32) / (2.0 * self.zoom);
        let half_h = (self.viewport_size.height as f32) / (2.0 * self.zoom);

        let left = -half_w;
        let right = half_w;
        let bottom = -half_h;
        let top = half_h;

        let proj = Matrix4::new_orthographic(left, right, bottom, top, -1.0, 1.0);
        let view = Matrix4::new_translation(&Vector3::new(-self.position.x, -self.position.y, 0.0));

        proj * view
    }

    pub fn screen_to_world(&self, screen_pos: LogicalPosition<f32>) -> Vector2<f32> {
        let screen_pos = Vector2::new(screen_pos.x, screen_pos.y);
        let screen_center = Vector2::new(
            self.viewport_size.width as f32 / 2.0,
            self.viewport_size.height as f32 / 2.0,
        );

        let screen_offset = screen_pos - screen_center;
        let mut world_offset = screen_offset / self.zoom;
        world_offset.y = -world_offset.y;

        self.position + world_offset
    }

    pub fn resize(&mut self, viewport_size: LogicalSize<u32>) {
        self.viewport_size = viewport_size;
    }

    pub fn update_lmb_state(&mut self, lmb_is_pressed: bool) {
        self.lmb_is_pressed = lmb_is_pressed;
    }

    pub fn update_cursor_position(&mut self, cursor_position: LogicalPosition<f32>) {
        if approx_eq!(f32, self.cursor_position.x, 0.)
            && approx_eq!(f32, self.cursor_position.y, 0.)
            && self.lmb_is_pressed
        {
            self.cursor_position = cursor_position;
            return;
        }

        if self.lmb_is_pressed {
            let old_x = self.cursor_position.x;
            let old_y = self.cursor_position.y;

            let new_x = cursor_position.x;
            let new_y = cursor_position.y;

            let delta_x = old_x - new_x;
            let delta_y = old_y - new_y;

            self.position.x += delta_x as f32;
            self.position.y -= delta_y as f32;
        }
        self.cursor_position = cursor_position;
    }
}
