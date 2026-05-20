use serde::{Deserialize, Serialize};
use crate::dof::Dof;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodalLoad {
    pub node_id: usize,
    pub dof: Dof,
    /// Magnitude in Newtons (forces) or Newton-metres (moments)
    pub magnitude: f64,
}

impl NodalLoad {
    pub fn new(node_id: usize, dof: Dof, magnitude: f64) -> Self {
        Self { node_id, dof, magnitude }
    }
}
