use nalgebra::{DMatrix, DVector, SMatrix, SVector};
use serde::{Deserialize, Serialize};

use crate::{
    dof::{Dof, global_dof_index},
    elements::traits::Element,
    load::{DistributedLoad, DistributedLoadDirection},
    material::Material,
    node::Node,
    section::Section,
};

pub type Matrix10 = SMatrix<f64, 10, 10>;
pub type Vector10 = SVector<f64, 10>;

/// A 3D Euler-Bernoulli beam aligned with the global X axis.
///
/// Active DOFs:
/// [u_iy, u_iz, rx_i, ry_i, rz_i, u_jy, u_jz, rx_j, ry_j, rz_j]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beam3D {
    pub id: usize,
    pub node_i: usize,
    pub node_j: usize,
    pub material: Material,
    pub section: Section,
}

impl Beam3D {
    pub fn new(
        id: usize,
        node_i: usize,
        node_j: usize,
        material: Material,
        section: Section,
    ) -> Self {
        assert_ne!(node_i, node_j, "Element endpoints must be different nodes");
        assert!(
            section.moment_of_inertia_y > 0.0,
            "Beam3D requires a positive local-y second moment of area"
        );
        assert!(
            section.moment_of_inertia_z > 0.0,
            "Beam3D requires a positive local-z second moment of area"
        );
        assert!(
            section.torsional_constant > 0.0,
            "Beam3D requires a positive torsional constant"
        );

        Self {
            id,
            node_i,
            node_j,
            material,
            section,
        }
    }

    pub fn geometry(&self, ni: &Node, nj: &Node) -> Beam3DGeometry {
        let dx = nj.x - ni.x;
        let dy = nj.y - ni.y;
        let dz = nj.z - ni.z;

        assert!(
            dy.abs() <= 1e-12 && dz.abs() <= 1e-12,
            "Beam3D element {} must be aligned with the global X axis",
            self.id
        );
        assert!(dx.abs() > 1e-12, "Element {} has zero length", self.id);

        Beam3DGeometry { length: dx.abs() }
    }

    pub fn local_stiffness_matrix(&self, nodes: &[Node]) -> Matrix10 {
        let geom = self.geometry(&nodes[self.node_i], &nodes[self.node_j]);

        let e = self.material.elastic_modulus;
        let g = self.material.shear_modulus();
        let iy = self.section.moment_of_inertia_y;
        let iz = self.section.moment_of_inertia_z;
        let j = self.section.torsional_constant;
        let l = geom.length;
        let l2 = l * l;
        let l3 = l2 * l;

        let gj_l = g * j / l;
        let eiy_l = e * iy / l;
        let eiz_l = e * iz / l;
        let eiy_l2 = e * iy / l2;
        let eiz_l2 = e * iz / l2;
        let eiy_l3 = e * iy / l3;
        let eiz_l3 = e * iz / l3;

        #[rustfmt::skip]
        let k = Matrix10::from_row_slice(&[
            12.0*eiz_l3,        0.0,    0.0,        0.0,  6.0*eiz_l2, -12.0*eiz_l3,       0.0,    0.0,        0.0,  6.0*eiz_l2,
                   0.0, 12.0*eiy_l3,   0.0, -6.0*eiy_l2,       0.0,        0.0, -12.0*eiy_l3,   0.0, -6.0*eiy_l2,       0.0,
                   0.0,        0.0,   gj_l,        0.0,        0.0,        0.0,        0.0,  -gj_l,       0.0,        0.0,
                   0.0, -6.0*eiy_l2,   0.0,  4.0*eiy_l,        0.0,        0.0,  6.0*eiy_l2,   0.0, 2.0*eiy_l,        0.0,
             6.0*eiz_l2,        0.0,    0.0,        0.0,  4.0*eiz_l, -6.0*eiz_l2,       0.0,    0.0,        0.0,  2.0*eiz_l,
           -12.0*eiz_l3,        0.0,    0.0,        0.0, -6.0*eiz_l2,  12.0*eiz_l3,      0.0,    0.0,        0.0, -6.0*eiz_l2,
                   0.0,-12.0*eiy_l3,   0.0,  6.0*eiy_l2,       0.0,        0.0,  12.0*eiy_l3,  0.0,  6.0*eiy_l2,       0.0,
                   0.0,        0.0,  -gj_l,        0.0,        0.0,        0.0,        0.0,   gj_l,       0.0,        0.0,
                   0.0, -6.0*eiy_l2,   0.0,  2.0*eiy_l,        0.0,        0.0,  6.0*eiy_l2,   0.0, 4.0*eiy_l,        0.0,
             6.0*eiz_l2,        0.0,    0.0,        0.0,  2.0*eiz_l, -6.0*eiz_l2,       0.0,    0.0,        0.0,  4.0*eiz_l,
        ]);

        k
    }

    pub fn end_forces(&self, nodes: &[Node], displacements: &[f64]) -> Vector10 {
        let dofs = self.dof_indices();
        let u = Vector10::from_iterator(dofs.iter().map(|&idx| displacements[idx]));

        self.local_stiffness_matrix(nodes) * u
    }

    pub fn equivalent_nodal_load_vector(
        &self,
        nodes: &[Node],
        distributed_loads: &[DistributedLoad],
    ) -> DVector<f64> {
        let l = self
            .geometry(&nodes[self.node_i], &nodes[self.node_j])
            .length;
        let mut f = Vector10::zeros();

        for load in distributed_loads
            .iter()
            .filter(|load| load.element_id == self.id)
        {
            let (wy, wz) = match load.direction {
                DistributedLoadDirection::LocalY | DistributedLoadDirection::GlobalY => {
                    (load.magnitude, 0.0)
                }
                DistributedLoadDirection::LocalZ | DistributedLoadDirection::GlobalZ => {
                    (0.0, load.magnitude)
                }
                DistributedLoadDirection::LocalX | DistributedLoadDirection::GlobalX => {
                    panic!(
                        "Beam3D element {} does not support distributed loads along the beam axis",
                        self.id
                    );
                }
            };

            #[rustfmt::skip]
            let fe = Vector10::from_row_slice(&[
                wy*l/2.0, wz*l/2.0,      0.0, -wz*l*l/12.0,  wy*l*l/12.0,
                wy*l/2.0, wz*l/2.0,      0.0,  wz*l*l/12.0, -wy*l*l/12.0,
            ]);

            f += fe;
        }

        DVector::from_row_slice(f.as_slice())
    }
}

impl Element for Beam3D {
    fn id(&self) -> usize {
        self.id
    }

    fn stiffness_matrix(&self, nodes: &[Node]) -> DMatrix<f64> {
        let k = self.local_stiffness_matrix(nodes);
        DMatrix::from_row_slice(10, 10, k.as_slice())
    }

    fn dof_indices(&self) -> Vec<usize> {
        vec![
            global_dof_index(self.node_i, Dof::Uy),
            global_dof_index(self.node_i, Dof::Uz),
            global_dof_index(self.node_i, Dof::Rx),
            global_dof_index(self.node_i, Dof::Ry),
            global_dof_index(self.node_i, Dof::Rz),
            global_dof_index(self.node_j, Dof::Uy),
            global_dof_index(self.node_j, Dof::Uz),
            global_dof_index(self.node_j, Dof::Rx),
            global_dof_index(self.node_j, Dof::Ry),
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
pub struct Beam3DGeometry {
    pub length: f64,
}
