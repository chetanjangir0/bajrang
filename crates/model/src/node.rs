use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub z: f64,
}

impl Node {
    pub fn new(id: usize, x: f64, y: f64) -> Self {
        Self::new_3d(id, x, y, 0.0)
    }

    pub fn new_3d(id: usize, x: f64, y: f64, z: f64) -> Self {
        Self { id, x, y, z }
    }

    pub fn position(&self) -> Vector3<f64> {
        Vector3::new(self.x, self.y, self.z)
    }
}
