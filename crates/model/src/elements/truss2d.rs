use serde::{Deserialize, Serialize};
use crate::{
    dof::{Dof, global_dof_index, DOFS_PER_NODE},
    material::Material,
    node::Node,
    section::Section,
};

/// A 2D truss element connecting two nodes.
///
/// Resists axial load only — no bending, no moment transfer.
/// Has 4 active DOFs(for global element stiffness): [u_ix, u_iy, u_jx, u_jy].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Truss2D {
    pub id: usize,
    pub node_i: usize,
    pub node_j: usize,
    pub material: Material,
    pub section: Section,
}

impl Truss2D {
    pub fn new(
        id: usize,
        node_i: usize,
        node_j: usize,
        material: Material,
        section: Section,
    ) -> Self {
        assert_ne!(node_i, node_j, "Element endpoints must be different nodes");
        Self { id, node_i, node_j, material, section }
    }

    pub fn geometry(&self, ni: &Node, nj: &Node) -> Truss2DGeometry {
        let dx = nj.x - ni.x;
        let dy = nj.y - ni.y;
        let length = (dx * dx + dy * dy).sqrt();

        assert!(length > 1e-12, "Element {} has zero length", self.id);

        Truss2DGeometry {
            length,
            cos: dx / length,
            sin: dy / length,
        }
    }

    pub fn global_dof_indices(&self) -> [usize; 4] {
        [
            global_dof_index(self.node_i, Dof::Ux),
            global_dof_index(self.node_i, Dof::Uy),
            global_dof_index(self.node_j, Dof::Ux),
            global_dof_index(self.node_j, Dof::Uy),
        ]
    }
}

/// Pre-computed geometric quantities for a truss element.
#[derive(Debug, Clone)]
pub struct Truss2DGeometry {
    pub length: f64,
    pub cos: f64,
    pub sin: f64,
}
