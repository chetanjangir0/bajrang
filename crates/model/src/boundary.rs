use crate::dof::Dof;
use serde::{Deserialize, Serialize};

/// support constraint: pins a specific DOF of a specific node to zero.
///
/// For a pin support on node 0: constrain Ux and Uy.
/// For a roller: constrain only Uy.
/// For a fixed 2D end: constrain Ux, Uy, and Rz.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Support {
    pub node_id: usize,
    pub dof: Dof,
}

impl Support {
    pub fn new(node_id: usize, dof: Dof) -> Self {
        Self { node_id, dof }
    }

    pub fn pin(node_id: usize) -> Vec<Support> {
        vec![
            Support::new(node_id, Dof::Ux),
            Support::new(node_id, Dof::Uy),
        ]
    }

    pub fn roller_y(node_id: usize) -> Vec<Support> {
        vec![Support::new(node_id, Dof::Uy)]
    }

    pub fn pin_3d(node_id: usize) -> Vec<Support> {
        vec![
            Support::new(node_id, Dof::Ux),
            Support::new(node_id, Dof::Uy),
            Support::new(node_id, Dof::Uz),
        ]
    }

    pub fn fixed_3d(node_id: usize) -> Vec<Support> {
        vec![
            Support::new(node_id, Dof::Ux),
            Support::new(node_id, Dof::Uy),
            Support::new(node_id, Dof::Uz),
            Support::new(node_id, Dof::Rx),
            Support::new(node_id, Dof::Ry),
            Support::new(node_id, Dof::Rz),
        ]
    }
}
