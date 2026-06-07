use crate::dof::Dof;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodalLoad {
    pub node_id: usize,
    pub dof: Dof,
    /// Magnitude in Newtons (forces) or Newton-metres (moments)
    pub magnitude: f64,
}

impl NodalLoad {
    pub fn new(node_id: usize, dof: Dof, magnitude: f64) -> Self {
        Self {
            node_id,
            dof,
            magnitude,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DistributedLoadDirection {
    LocalX,
    LocalY,
    GlobalX,
    GlobalY,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DistributedLoad {
    pub element_id: usize,
    /// Force intensity per unit length.
    pub magnitude: f64,
    pub direction: DistributedLoadDirection,
}

impl DistributedLoad {
    pub fn local_x(element_id: usize, magnitude: f64) -> Self {
        Self {
            element_id,
            magnitude,
            direction: DistributedLoadDirection::LocalX,
        }
    }

    pub fn local_y(element_id: usize, magnitude: f64) -> Self {
        Self {
            element_id,
            magnitude,
            direction: DistributedLoadDirection::LocalY,
        }
    }

    pub fn global_x(element_id: usize, magnitude: f64) -> Self {
        Self {
            element_id,
            magnitude,
            direction: DistributedLoadDirection::GlobalX,
        }
    }

    pub fn global_y(element_id: usize, magnitude: f64) -> Self {
        Self {
            element_id,
            magnitude,
            direction: DistributedLoadDirection::GlobalY,
        }
    }
}
