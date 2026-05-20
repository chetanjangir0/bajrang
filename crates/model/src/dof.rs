use serde::{Serialize, Deserialize};

/// The global DOF index for node `i` (in the global stiffness matrix) with DOF `d` is:
///   global_index = i * DOFS_PER_NODE + d as usize
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dof {
    /// Translation in X
    Ux = 0,
    /// Translation in Y
    Uy = 1,
    /// Rotation about Z
    Rz = 2,
}

pub const DOFS_PER_NODE: usize = 3;

pub fn global_dof_index(node_id: usize, dof: Dof) -> usize {
    node_id * DOFS_PER_NODE + dof as usize
}
