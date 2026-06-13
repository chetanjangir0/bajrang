use nalgebra::{DMatrix, DVector, Matrix6, Vector6};
use serde::{Deserialize, Serialize};

use crate::{
    dof::{Dof, global_dof_index},
    elements::traits::Element,
    load::{DistributedLoad, DistributedLoadDirection},
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

    pub fn equivalent_nodal_load_vector(
        &self,
        nodes: &[Node],
        distributed_loads: &[DistributedLoad],
    ) -> DVector<f64> {
        let geom = self.geometry(&nodes[self.node_i], &nodes[self.node_j]);
        let l = geom.length;
        let t = self.transformation_matrix(nodes);
        let mut f_local = Vector6::zeros();

        for load in distributed_loads
            .iter()
            .filter(|load| load.element_id == self.id)
        {
            let (wx_local, wy_local) = match load.direction {
                DistributedLoadDirection::LocalX => (load.magnitude, 0.0),
                DistributedLoadDirection::LocalY => (0.0, load.magnitude),
                DistributedLoadDirection::GlobalX => {
                    (geom.cos * load.magnitude, -geom.sin * load.magnitude)
                }
                DistributedLoadDirection::GlobalY => {
                    (geom.sin * load.magnitude, geom.cos * load.magnitude)
                }
                DistributedLoadDirection::LocalZ | DistributedLoadDirection::GlobalZ => {
                    panic!(
                        "Frame2D element {} does not support out-of-plane distributed loads",
                        self.id
                    );
                }
            };

            #[rustfmt::skip]
            let fe_axial = Vector6::from_row_slice(&[
                wx_local * l / 2.0,
                0.0,
                0.0,
                wx_local * l / 2.0,
                0.0,
                0.0,
            ]);

            #[rustfmt::skip]
            let fe_transverse = Vector6::from_row_slice(&[
                0.0,
                wy_local * l / 2.0,
                wy_local * l * l / 12.0,
                0.0,
                wy_local * l / 2.0,
                -wy_local * l * l / 12.0,
            ]);

            f_local += fe_axial + fe_transverse;
        }

        let f_global = t.transpose() * f_local;
        DVector::from_row_slice(f_global.as_slice())
    }
}

impl Element for Frame2D {
    fn id(&self) -> usize {
        self.id
    }

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

    fn equivalent_load_vector(
        &self,
        nodes: &[Node],
        distributed_loads: &[DistributedLoad],
    ) -> DVector<f64> {
        self.equivalent_nodal_load_vector(nodes, distributed_loads)
    }
}

#[derive(Debug, Clone)]
pub struct Frame2DGeometry {
    pub length: f64,
    pub cos: f64,
    pub sin: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{load::DistributedLoad, material::Material, node::Node, section::Section};

    fn make_frame() -> (Vec<Node>, Frame2D) {
        let nodes = vec![Node::new(0, 0.0, 0.0), Node::new(1, 3.0, 4.0)];
        let frame = Frame2D::new(
            11,
            0,
            1,
            Material::new(210.0e9, 0.3),
            Section::new(0.02, 1.6e-5),
        );

        (nodes, frame)
    }

    fn assert_close(actual: f64, expected: f64, tol: f64, label: &str) {
        assert!(
            (actual - expected).abs() <= tol,
            "{label}: expected {expected:.12e}, got {actual:.12e}"
        );
    }

    #[test]
    fn transformation_matrix_is_orthonormal() {
        let (nodes, frame) = make_frame();
        let t = frame.transformation_matrix(&nodes);
        let identity = t * t.transpose();

        for row in 0..6 {
            for col in 0..6 {
                let expected = if row == col { 1.0 } else { 0.0 };
                assert_close(identity[(row, col)], expected, 1e-12, "Orthogonality");
            }
        }
    }

    #[test]
    fn horizontal_frame_global_stiffness_matches_local_stiffness() {
        let nodes = vec![Node::new(0, 0.0, 0.0), Node::new(1, 2.0, 0.0)];
        let frame = Frame2D::new(
            5,
            0,
            1,
            Material::new(210.0e9, 0.3),
            Section::new(0.02, 1.6e-5),
        );

        let local = frame.local_stiffness_matrix(&nodes);
        let global = frame.stiffness_matrix(&nodes);

        for row in 0..6 {
            for col in 0..6 {
                assert_close(
                    global[(row, col)],
                    local[(row, col)],
                    1e-12,
                    "Horizontal stiffness",
                );
            }
        }
    }

    #[test]
    fn uniform_local_y_load_rotates_into_global_coordinates() {
        let (nodes, frame) = make_frame();
        let loads = vec![DistributedLoad::local_y(11, -2.0)];

        let fe = frame.equivalent_nodal_load_vector(&nodes, &loads);

        assert_close(fe[0], 4.0, 1e-12, "Node i global X");
        assert_close(fe[1], -3.0, 1e-12, "Node i global Y");
        assert_close(fe[2], -25.0 / 6.0, 1e-12, "Node i moment");
        assert_close(fe[3], 4.0, 1e-12, "Node j global X");
        assert_close(fe[4], -3.0, 1e-12, "Node j global Y");
        assert_close(fe[5], 25.0 / 6.0, 1e-12, "Node j moment");
    }

    #[test]
    fn uniform_global_y_load_converts_to_global_equivalent_nodal_forces() {
        let (nodes, frame) = make_frame();
        let loads = vec![DistributedLoad::global_y(11, -2.0)];

        let fe = frame.equivalent_nodal_load_vector(&nodes, &loads);

        assert_close(fe[0], 0.0, 1e-12, "Node i global X");
        assert_close(fe[1], -5.0, 1e-12, "Node i global Y");
        assert_close(fe[2], -2.5, 1e-12, "Node i moment");
        assert_close(fe[3], 0.0, 1e-12, "Node j global X");
        assert_close(fe[4], -5.0, 1e-12, "Node j global Y");
        assert_close(fe[5], 2.5, 1e-12, "Node j moment");
    }

    #[test]
    fn uniform_local_x_load_creates_axial_equivalent_nodal_forces() {
        let (nodes, frame) = make_frame();
        let loads = vec![DistributedLoad::local_x(11, 3.0)];

        let fe = frame.equivalent_nodal_load_vector(&nodes, &loads);

        assert_close(fe[0], 4.5, 1e-12, "Node i global X");
        assert_close(fe[1], 6.0, 1e-12, "Node i global Y");
        assert_close(fe[2], 0.0, 1e-12, "Node i moment");
        assert_close(fe[3], 4.5, 1e-12, "Node j global X");
        assert_close(fe[4], 6.0, 1e-12, "Node j global Y");
        assert_close(fe[5], 0.0, 1e-12, "Node j moment");
    }
}
