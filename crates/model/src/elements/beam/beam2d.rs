use nalgebra::{DMatrix, DVector, Vector4};
use serde::{Deserialize, Serialize};

use crate::{
    dof::{Dof, global_dof_index},
    elements::traits::Element,
    load::{DistributedLoad, DistributedLoadDirection},
    material::Material,
    node::Node,
    section::Section,
};

/// A 2D Euler-Bernoulli beam aligned with the global X axis.
///
/// Active DOFs:
/// [u_iy, rz_i, u_jy, rz_j]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beam2D {
    pub id: usize,
    pub node_i: usize,
    pub node_j: usize,
    pub material: Material,
    pub section: Section,
}

impl Beam2D {
    pub fn new(
        id: usize,
        node_i: usize,
        node_j: usize,
        material: Material,
        section: Section,
    ) -> Self {
        assert_ne!(node_i, node_j, "Element endpoints must be different nodes");
        assert!(
            section.moment_of_inertia > 0.0,
            "Beam2D requires a positive second moment of area"
        );

        Self {
            id,
            node_i,
            node_j,
            material,
            section,
        }
    }

    pub fn geometry(&self, ni: &Node, nj: &Node) -> Beam2DGeometry {
        let dx = nj.x - ni.x;
        let dy = nj.y - ni.y;

        assert!(
            dy.abs() <= 1e-12,
            "Beam2D element {} must be aligned with the global X axis",
            self.id
        );
        assert!(dx.abs() > 1e-12, "Element {} has zero length", self.id);

        Beam2DGeometry { length: dx.abs() }
    }

    pub fn local_stiffness_matrix(&self, nodes: &[Node]) -> DMatrix<f64> {
        let ni = &nodes[self.node_i];
        let nj = &nodes[self.node_j];
        let geom = self.geometry(ni, nj);
        let ei = self.material.elastic_modulus * self.section.moment_of_inertia;
        let l = geom.length;

        let l2 = l * l;
        let l3 = l2 * l;
        let scale = ei / l3;

        #[rustfmt::skip]
        let k = DMatrix::from_row_slice(4, 4, &[
             12.0,   6.0*l, -12.0,   6.0*l,
             6.0*l,  4.0*l2, -6.0*l,  2.0*l2,
            -12.0,  -6.0*l,  12.0,  -6.0*l,
             6.0*l,  2.0*l2, -6.0*l,  4.0*l2,
        ]);

        scale * k
    }

    pub fn end_forces(&self, nodes: &[Node], displacements: &[f64]) -> Vector4<f64> {
        let dofs = self.dof_indices();
        let u = DVector::from_iterator(dofs.len(), dofs.iter().map(|&idx| displacements[idx]));
        let forces = self.local_stiffness_matrix(nodes) * u;

        Vector4::new(forces[0], forces[1], forces[2], forces[3])
    }

    pub fn equivalent_nodal_load_vector(
        &self,
        nodes: &[Node],
        distributed_loads: &[DistributedLoad],
    ) -> DVector<f64> {
        let l = self
            .geometry(&nodes[self.node_i], &nodes[self.node_j])
            .length;
        let mut f = DVector::zeros(4);

        for load in distributed_loads
            .iter()
            .filter(|load| load.element_id == self.id)
        {
            match load.direction {
                DistributedLoadDirection::LocalY | DistributedLoadDirection::GlobalY => {
                    #[rustfmt::skip]
                    let fe = DVector::from_row_slice(&[
                        load.magnitude * l / 2.0,
                        load.magnitude * l * l / 12.0,
                        load.magnitude * l / 2.0,
                        -load.magnitude * l * l / 12.0,
                    ]);

                    f += fe;
                }
                DistributedLoadDirection::LocalX | DistributedLoadDirection::GlobalX => {
                    panic!(
                        "Beam2D element {} does not support distributed loads along the beam axis",
                        self.id
                    );
                }
                DistributedLoadDirection::LocalZ | DistributedLoadDirection::GlobalZ => {
                    panic!(
                        "Beam2D element {} does not support out-of-plane distributed loads",
                        self.id
                    );
                }
            }
        }

        f
    }
}

impl Element for Beam2D {
    fn id(&self) -> usize {
        self.id
    }

    fn stiffness_matrix(&self, nodes: &[Node]) -> DMatrix<f64> {
        self.local_stiffness_matrix(nodes)
    }

    fn dof_indices(&self) -> Vec<usize> {
        vec![
            global_dof_index(self.node_i, Dof::Uy),
            global_dof_index(self.node_i, Dof::Rz),
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
pub struct Beam2DGeometry {
    pub length: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{load::DistributedLoad, material::Material, node::Node, section::Section};

    fn make_beam() -> (Vec<Node>, Beam2D) {
        let nodes = vec![Node::new(0, 0.0, 0.0), Node::new(1, 3.0, 0.0)];
        let beam = Beam2D::new(
            7,
            0,
            1,
            Material::new(200.0e9, 0.3),
            Section::new(0.02, 8.0e-6),
        );

        (nodes, beam)
    }

    fn assert_close(actual: f64, expected: f64, tol: f64, label: &str) {
        assert!(
            (actual - expected).abs() <= tol,
            "{label}: expected {expected:.12e}, got {actual:.12e}"
        );
    }

    #[test]
    fn uniform_local_y_load_maps_to_consistent_nodal_vector() {
        let (nodes, beam) = make_beam();
        let loads = vec![DistributedLoad::local_y(7, -2.0)];

        let fe = beam.equivalent_nodal_load_vector(&nodes, &loads);

        assert_close(fe[0], -3.0, 1e-12, "Node i shear");
        assert_close(fe[1], -1.5, 1e-12, "Node i moment");
        assert_close(fe[2], -3.0, 1e-12, "Node j shear");
        assert_close(fe[3], 1.5, 1e-12, "Node j moment");
    }

    #[test]
    fn uniform_global_y_load_maps_to_same_consistent_nodal_vector() {
        let (nodes, beam) = make_beam();
        let loads = vec![DistributedLoad::global_y(7, -2.0)];

        let fe = beam.equivalent_nodal_load_vector(&nodes, &loads);

        assert_close(fe[0], -3.0, 1e-12, "Node i shear");
        assert_close(fe[1], -1.5, 1e-12, "Node i moment");
        assert_close(fe[2], -3.0, 1e-12, "Node j shear");
        assert_close(fe[3], 1.5, 1e-12, "Node j moment");
    }

    #[test]
    fn ignores_distributed_loads_for_other_elements() {
        let (nodes, beam) = make_beam();
        let loads = vec![DistributedLoad::local_y(99, -2.0)];

        let fe = beam.equivalent_nodal_load_vector(&nodes, &loads);

        assert_eq!(fe, DVector::zeros(4));
    }
}
