use nalgebra::{DMatrix, DVector, Matrix6, Vector6};
use serde::{Deserialize, Serialize};

use crate::{
    dof::{Dof, global_dof_index},
    elements::traits::Element,
    material::Material,
    node::Node,
    section::Section,
};

/// A 2D frame element with axial and flexural stiffness.
///
/// Active DOFs:
/// [u_ix, u_iy, rz_i, u_jx, u_jy, rz_j]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame2D {
    pub id: usize,
    pub node_i: usize,
    pub node_j: usize,
    pub material: Material,
    pub section: Section,
}

impl Frame2D {
    pub fn new(
        id: usize,
        node_i: usize,
        node_j: usize,
        material: Material,
        section: Section,
    ) -> Self {
        assert_ne!(node_i, node_j, "Element endpoints must be different nodes");
        assert!(section.area > 0.0, "Frame2D requires a positive area");
        assert!(
            section.moment_of_inertia > 0.0,
            "Frame2D requires a positive second moment of area"
        );

        Self {
            id,
            node_i,
            node_j,
            material,
            section,
        }
    }

    pub fn geometry(&self, ni: &Node, nj: &Node) -> Frame2DGeometry {
        let dx = nj.x - ni.x;
        let dy = nj.y - ni.y;
        let length = (dx * dx + dy * dy).sqrt();

        assert!(length > 1e-12, "Element {} has zero length", self.id);

        Frame2DGeometry {
            length,
            cos: dx / length,
            sin: dy / length,
        }
    }

    pub fn local_stiffness_matrix(&self, nodes: &[Node]) -> Matrix6<f64> {
        let ni = &nodes[self.node_i];
        let nj = &nodes[self.node_j];
        let geom = self.geometry(ni, nj);

        let e = self.material.elastic_modulus;
        let a = self.section.area;
        let i = self.section.moment_of_inertia;
        let l = geom.length;
        let l2 = l * l;
        let l3 = l2 * l;

        let ea_over_l = e * a / l;
        let ei_over_l3 = e * i / l3;

        #[rustfmt::skip]
        let k = Matrix6::from_row_slice(&[
             ea_over_l,          0.0,            0.0, -ea_over_l,          0.0,            0.0,
                  0.0,  12.0*ei_over_l3,  6.0*l*ei_over_l3,       0.0, -12.0*ei_over_l3,  6.0*l*ei_over_l3,
                  0.0,  6.0*l*ei_over_l3, 4.0*l2*ei_over_l3,      0.0, -6.0*l*ei_over_l3, 2.0*l2*ei_over_l3,
            -ea_over_l,          0.0,            0.0,  ea_over_l,          0.0,            0.0,
                  0.0, -12.0*ei_over_l3, -6.0*l*ei_over_l3,       0.0,  12.0*ei_over_l3, -6.0*l*ei_over_l3,
                  0.0,  6.0*l*ei_over_l3, 2.0*l2*ei_over_l3,      0.0, -6.0*l*ei_over_l3, 4.0*l2*ei_over_l3,
        ]);

        k
    }

    pub fn transformation_matrix(&self, nodes: &[Node]) -> Matrix6<f64> {
        let ni = &nodes[self.node_i];
        let nj = &nodes[self.node_j];
        let geom = self.geometry(ni, nj);
        let c = geom.cos;
        let s = geom.sin;

        #[rustfmt::skip]
        let t = Matrix6::from_row_slice(&[
             c,  s, 0.0, 0.0, 0.0, 0.0,
            -s,  c, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0,  c,  s, 0.0,
            0.0, 0.0, 0.0, -s,  c, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]);

        t
    }

    pub fn end_forces(&self, nodes: &[Node], displacements: &[f64]) -> Vector6<f64> {
        let dofs = self.dof_indices();
        let u_global = Vector6::from_iterator(dofs.iter().map(|&idx| displacements[idx]));
        let u_local = self.transformation_matrix(nodes) * u_global;

        self.local_stiffness_matrix(nodes) * u_local
    }
}

impl Element for Frame2D {
    fn stiffness_matrix(&self, nodes: &[Node]) -> DMatrix<f64> {
        let k_local = self.local_stiffness_matrix(nodes);
        let t = self.transformation_matrix(nodes);
        let k_global = t.transpose() * k_local * t;

        DMatrix::from_row_slice(6, 6, k_global.as_slice())
    }

    fn dof_indices(&self) -> Vec<usize> {
        vec![
            global_dof_index(self.node_i, Dof::Ux),
            global_dof_index(self.node_i, Dof::Uy),
            global_dof_index(self.node_i, Dof::Rz),
            global_dof_index(self.node_j, Dof::Ux),
            global_dof_index(self.node_j, Dof::Uy),
            global_dof_index(self.node_j, Dof::Rz),
        ]
    }

    fn equivalent_load_vector(&self, _nodes: &[Node]) -> DVector<f64> {
        DVector::zeros(6)
    }
}

#[derive(Debug, Clone)]
pub struct Frame2DGeometry {
    pub length: f64,
    pub cos: f64,
    pub sin: f64,
}
