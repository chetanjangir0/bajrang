use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dof {
    Ux = 0,
    Uy = 1,
    Uz = 2,
    Rx = 3,
    Ry = 4,
    Rz = 5,
}

pub const DOFS_PER_NODE: usize = 6;

pub fn global_dof_index(node_id: usize, dof: Dof) -> usize {
    node_id * DOFS_PER_NODE + dof as usize
}
