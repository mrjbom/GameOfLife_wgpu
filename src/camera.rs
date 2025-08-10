// 2D Camera

use float_cmp::approx_eq;
use nalgebra::{Matrix4, Vector2, Vector3};
use winit::dpi::{LogicalPosition, LogicalSize};

const ZOOM_MIN: f32 = 0.1;
const ZOOM_MAX: f32 = 10.;

const ZOOM_DEFAULT_SENSITIVITY: f32 = 0.1;

#[derive(Debug)]
pub struct Camera {
    position: Vector2<f32>,
    zoom: f32,
    zoom_sensitivity: f32,
    viewport_size: LogicalSize<u32>,
    lmb_is_pressed: bool,
    cursor_position: LogicalPosition<f32>,
}

impl Camera {
    pub fn new(viewport_size: LogicalSize<u32>) -> Self {
        Self {
            position: Vector2::new(0., 0.),
            zoom: 1.,
            zoom_sensitivity: ZOOM_DEFAULT_SENSITIVITY,
            viewport_size,
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

    pub fn mouse_scroll(&mut self, scroll: f32) {
        let zoom_delta = scroll * self.zoom_sensitivity;
        let old_zoom = self.zoom;
        let new_zoom = (old_zoom + zoom_delta).clamp(ZOOM_MIN, ZOOM_MAX);
        if approx_eq!(f32, old_zoom, new_zoom) {
            return;
        }

        // Zoom changed
        // For camera zoom on the cursor, we need to move its position
        let cursor_position = self.cursor_position;
        let old_cursor_world_position = self.screen_to_world_position(cursor_position);
        self.zoom = new_zoom;
        let new_cursor_world_position = self.screen_to_world_position(cursor_position);
        let cursor_position_delta = new_cursor_world_position - old_cursor_world_position;
        self.position -= cursor_position_delta;
    }

    pub fn set_zoom_sensitivity(&mut self, zoom_sensitivity: f32) {
        self.zoom_sensitivity = zoom_sensitivity;
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

    pub fn screen_to_world_position(&self, screen_pos: LogicalPosition<f32>) -> Vector2<f32> {
        let screen_position = Vector2::new(screen_pos.x, screen_pos.y);
        let screen_center = Vector2::new(
            self.viewport_size.width as f32 / 2.0,
            self.viewport_size.height as f32 / 2.0,
        );

        let screen_offset = screen_position - screen_center;
        let mut world_offset = screen_offset / self.zoom;
        world_offset.y = -world_offset.y;

        self.position + world_offset
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

            self.position.x += delta_x / self.zoom;
            self.position.y -= delta_y / self.zoom;
        }
        self.cursor_position = cursor_position;
    }
}
