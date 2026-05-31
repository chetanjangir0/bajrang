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
        let l = self
            .geometry(&nodes[self.node_i], &nodes[self.node_j])
            .length;
        let mut f = DVector::zeros(4);

        for load in distributed_loads
            .iter()
            .filter(|load| load.element_id == self.id)
        {
            match load.direction {
                DistributedLoadDirection::LocalY => {
                    #[rustfmt::skip]
                    let fe = DVector::from_row_slice(&[
                        load.magnitude * l / 2.0,
                        load.magnitude * l * l / 12.0,
                        load.magnitude * l / 2.0,
                        -load.magnitude * l * l / 12.0,
                    ]);

                    f += fe;
                }
            }
        }

        f
    }
}

#[derive(Debug, Clone)]
pub struct Beam2DGeometry {
    pub length: f64,
}
