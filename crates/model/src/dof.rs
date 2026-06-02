use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dof {
    Ux = 0,
    Uy = 1,
    Rz = 2,
}

pub const DOFS_PER_NODE: usize = 3;

pub fn global_dof_index(node_id: usize, dof: Dof) -> usize {
    node_id * DOFS_PER_NODE + dof as usize
}
