use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: usize,
    pub x: f64,
    pub y: f64,
}

impl Node {
    pub fn new(id: usize, x: f64, y: f64) -> Self {
        Self { id, x, y }
    }

    pub fn position(&self) -> Vector2<f64> {
        Vector2::new(self.x, self.y)
    }
}
