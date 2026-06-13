#[derive(Debug, Clone, Copy)]
pub struct ViewportState {
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            zoom: 54.0,
            pan_x: 0.0,
            pan_y: 18.0,
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
            ViewportUpdate::Zoom {
                factor,
                pivot_x,
                pivot_y,
            } => {
                let old_zoom = self.zoom;
                self.zoom = (self.zoom * factor).clamp(18.0, 220.0);
                let applied = self.zoom / old_zoom;

                self.pan_x = pivot_x - (pivot_x - self.pan_x) * applied;
                self.pan_y = pivot_y - (pivot_y - self.pan_y) * applied;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ViewportUpdate {
    Pan {
        dx: f32,
        dy: f32,
    },
    Zoom {
        factor: f32,
        pivot_x: f32,
        pivot_y: f32,
    },
}

#[derive(Debug, Clone)]
pub enum ViewportEvent {
    Selected(Option<crate::state::Selection>),
    Changed(ViewportUpdate),
}
