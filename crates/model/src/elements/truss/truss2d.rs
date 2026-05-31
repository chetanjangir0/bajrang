use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};

use crate::{
    dof::{Dof, global_dof_index},
    elements::traits::Element,
    material::Material,
    node::Node,
    section::Section,
};

/// A 2D truss element connecting two nodes.
///
/// Resists axial load only — no bending, no moment transfer.
/// Has 4 active DOFs for global element stiffness:
/// [u_ix, u_iy, u_jx, u_jy].
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
        Self {
            id,
            node_i,
            node_j,
            material,
            section,
        }
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

    pub fn axial_force(&self, nodes: &[Node], displacements: &[f64]) -> f64 {
        let ni = &nodes[self.node_i];
        let nj = &nodes[self.node_j];
        let geom = self.geometry(ni, nj);
        let ae_over_l = self.material.elastic_modulus * self.section.area / geom.length;

        let c = geom.cos;
        let s = geom.sin;

        let dofs = self.dof_indices();
        let u_ix = displacements[dofs[0]];
        let u_iy = displacements[dofs[1]];
        let u_jx = displacements[dofs[2]];
        let u_jy = displacements[dofs[3]];

        ae_over_l * (c * (u_jx - u_ix) + s * (u_jy - u_iy))
    }
}

impl Element for Truss2D {
    fn stiffness_matrix(&self, nodes: &[Node]) -> DMatrix<f64> {
        let ni = &nodes[self.node_i];
        let nj = &nodes[self.node_j];
        let geom = self.geometry(ni, nj);
        let ae_over_l = self.material.elastic_modulus * self.section.area / geom.length;

        let c = geom.cos;
        let s = geom.sin;

        #[rustfmt::skip]
        let k = DMatrix::from_row_slice(4, 4, &[
             c*c,  c*s, -c*c, -c*s,
             c*s,  s*s, -c*s, -s*s,
            -c*c, -c*s,  c*c,  c*s,
            -c*s, -s*s,  c*s,  s*s,
        ]);

        ae_over_l * k
    }

    fn dof_indices(&self) -> Vec<usize> {
        vec![
            global_dof_index(self.node_i, Dof::Ux),
            global_dof_index(self.node_i, Dof::Uy),
            global_dof_index(self.node_j, Dof::Ux),
            global_dof_index(self.node_j, Dof::Uy),
        ]
    }

    fn equivalent_load_vector(&self, _nodes: &[Node]) -> DVector<f64> {
        DVector::zeros(4)
    }
}

/// Pre-computed geometric quantities for a truss element.
#[derive(Debug, Clone)]
pub struct Truss2DGeometry {
    pub length: f64,
    pub cos: f64,
    pub sin: f64,
}
