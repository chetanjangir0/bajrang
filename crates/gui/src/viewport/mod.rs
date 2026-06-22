use iced::{Point, Rectangle};

use crate::state::Selection;

pub const DEFAULT_ZOOM: f32 = 58.0;

#[derive(Debug, Clone, Copy)]
pub struct ViewportState {
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            zoom: DEFAULT_ZOOM,
            pan_x: 0.0,
            pan_y: 20.0,
        }
    }
}

impl ViewportState {
    pub fn apply(&mut self, update: ViewportUpdate) {
        match update {
            ViewportUpdate::Pan { dx, dy } => {
                self.pan_x += dx;
                self.pan_y += dy;
            }
            ViewportUpdate::Zoom { factor, pivot } => {
                let old_zoom = self.zoom;
                self.zoom = (self.zoom * factor).clamp(16.0, 260.0);
                let applied = self.zoom / old_zoom;

                self.pan_x = pivot.x - (pivot.x - self.pan_x) * applied;
                self.pan_y = pivot.y - (pivot.y - self.pan_y) * applied;
            }
        }
    }

    pub fn model_to_screen(self, x: f64, y: f64, bounds: Rectangle) -> Point {
        let origin = self.origin(bounds);

        Point::new(
            origin.x + x as f32 * self.zoom,
            origin.y - y as f32 * self.zoom,
        )
    }

    pub fn screen_to_model(self, point: Point, bounds: Rectangle) -> (f64, f64) {
        let origin = self.origin(bounds);

        (
            ((point.x - origin.x) / self.zoom) as f64,
            ((origin.y - point.y) / self.zoom) as f64,
        )
    }

    pub fn origin(self, bounds: Rectangle) -> Point {
        Point::new(
            bounds.width * 0.5 + self.pan_x,
            bounds.height * 0.64 + self.pan_y,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ViewportUpdate {
    Pan { dx: f32, dy: f32 },
    Zoom { factor: f32, pivot: Point },
}

#[derive(Debug, Clone, Copy)]
pub struct ViewportPress {
    pub target: Option<Selection>,
    pub model_x: f64,
    pub model_y: f64,
}

#[derive(Debug, Clone)]
pub enum ViewportEvent {
    Pressed(ViewportPress),
    Changed(ViewportUpdate),
}
