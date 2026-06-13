use nalgebra::{DMatrix, DVector, Matrix3, SMatrix, SVector, Vector3};
use serde::{Deserialize, Serialize};

use crate::{
    dof::{Dof, global_dof_index},
    elements::traits::Element,
    load::{DistributedLoad, DistributedLoadDirection},
    material::Material,
    node::Node,
    section::Section,
};

pub type Matrix12 = SMatrix<f64, 12, 12>;
pub type Vector12 = SVector<f64, 12>;

/// A 3D frame element with axial, torsional, and biaxial flexural stiffness.
///
/// Active DOFs:
/// [u_ix, u_iy, u_iz, rx_i, ry_i, rz_i, u_jx, u_jy, u_jz, rx_j, ry_j, rz_j]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame3D {
    pub id: usize,
    pub node_i: usize,
    pub node_j: usize,
    pub material: Material,
    pub section: Section,
}

impl Frame3D {
    pub fn new(
        id: usize,
        node_i: usize,
        node_j: usize,
        material: Material,
        section: Section,
    ) -> Self {
        assert_ne!(node_i, node_j, "Element endpoints must be different nodes");
        assert!(section.area > 0.0, "Frame3D requires a positive area");
        assert!(
            section.moment_of_inertia_y > 0.0,
            "Frame3D requires a positive local-y second moment of area"
        );
        assert!(
            section.moment_of_inertia_z > 0.0,
            "Frame3D requires a positive local-z second moment of area"
        );
        assert!(
            section.torsional_constant > 0.0,
            "Frame3D requires a positive torsional constant"
        );

        Self {
            id,
            node_i,
            node_j,
            material,
            section,
        }
    }

    pub fn geometry(&self, ni: &Node, nj: &Node) -> Frame3DGeometry {
        let delta = nj.position() - ni.position();
        let length = delta.norm();

        assert!(length > 1e-12, "Element {} has zero length", self.id);

        let local_x = delta / length;
        let reference = if local_x.cross(&Vector3::z()).norm() > 1e-12 {
            Vector3::z()
        } else {
            Vector3::y()
        };
        let local_y = reference.cross(&local_x).normalize();
        let local_z = local_x.cross(&local_y).normalize();

        Frame3DGeometry {
            length,
            local_x,
            local_y,
            local_z,
        }
    }

    pub fn local_stiffness_matrix(&self, nodes: &[Node]) -> Matrix12 {
        let geom = self.geometry(&nodes[self.node_i], &nodes[self.node_j]);

        let e = self.material.elastic_modulus;
        let g = self.material.shear_modulus();
        let a = self.section.area;
        let iy = self.section.moment_of_inertia_y;
        let iz = self.section.moment_of_inertia_z;
        let j = self.section.torsional_constant;
        let l = geom.length;
        let l2 = l * l;
        let l3 = l2 * l;

        let ea_l = e * a / l;
        let gj_l = g * j / l;
        let eiy_l = e * iy / l;
        let eiz_l = e * iz / l;
        let eiy_l2 = e * iy / l2;
        let eiz_l2 = e * iz / l2;
        let eiy_l3 = e * iy / l3;
        let eiz_l3 = e * iz / l3;

        #[rustfmt::skip]
        let k = Matrix12::from_row_slice(&[
             ea_l,        0.0,        0.0,    0.0,        0.0,        0.0, -ea_l,        0.0,        0.0,    0.0,        0.0,        0.0,
              0.0, 12.0*eiz_l3,       0.0,    0.0,        0.0,  6.0*eiz_l2,   0.0,-12.0*eiz_l3,       0.0,    0.0,        0.0,  6.0*eiz_l2,
              0.0,        0.0, 12.0*eiy_l3,   0.0, -6.0*eiy_l2,       0.0,   0.0,        0.0,-12.0*eiy_l3,   0.0, -6.0*eiy_l2,       0.0,
              0.0,        0.0,        0.0,   gj_l,        0.0,        0.0,   0.0,        0.0,        0.0,  -gj_l,        0.0,        0.0,
              0.0,        0.0, -6.0*eiy_l2,   0.0,  4.0*eiy_l,        0.0,   0.0,        0.0,  6.0*eiy_l2,   0.0,  2.0*eiy_l,        0.0,
              0.0,  6.0*eiz_l2,       0.0,    0.0,        0.0,  4.0*eiz_l,   0.0, -6.0*eiz_l2,       0.0,    0.0,        0.0,  2.0*eiz_l,
            -ea_l,        0.0,        0.0,    0.0,        0.0,        0.0,  ea_l,        0.0,        0.0,    0.0,        0.0,        0.0,
              0.0,-12.0*eiz_l3,       0.0,    0.0,        0.0, -6.0*eiz_l2,   0.0, 12.0*eiz_l3,       0.0,    0.0,        0.0, -6.0*eiz_l2,
              0.0,        0.0,-12.0*eiy_l3,   0.0,  6.0*eiy_l2,       0.0,   0.0,        0.0, 12.0*eiy_l3,   0.0,  6.0*eiy_l2,       0.0,
              0.0,        0.0,        0.0,  -gj_l,        0.0,        0.0,   0.0,        0.0,        0.0,   gj_l,        0.0,        0.0,
              0.0,        0.0, -6.0*eiy_l2,   0.0,  2.0*eiy_l,        0.0,   0.0,        0.0,  6.0*eiy_l2,   0.0,  4.0*eiy_l,        0.0,
              0.0,  6.0*eiz_l2,       0.0,    0.0,        0.0,  2.0*eiz_l,   0.0, -6.0*eiz_l2,       0.0,    0.0,        0.0,  4.0*eiz_l,
        ]);

        k
    }

    pub fn transformation_matrix(&self, nodes: &[Node]) -> Matrix12 {
        let geom = self.geometry(&nodes[self.node_i], &nodes[self.node_j]);
        let rotation = Matrix3::from_rows(&[
            geom.local_x.transpose(),
            geom.local_y.transpose(),
            geom.local_z.transpose(),
        ]);

        let mut t = Matrix12::zeros();
        for block in [0, 3, 6, 9] {
            t.fixed_view_mut::<3, 3>(block, block).copy_from(&rotation);
        }

        t
    }

    pub fn end_forces(&self, nodes: &[Node], displacements: &[f64]) -> Vector12 {
        let dofs = self.dof_indices();
        let u_global = Vector12::from_iterator(dofs.iter().map(|&idx| displacements[idx]));
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
        let mut f_local = Vector12::zeros();

        for load in distributed_loads
            .iter()
            .filter(|load| load.element_id == self.id)
        {
            let global = |direction: Vector3<f64>| -> (f64, f64, f64) {
                let local = Vector3::new(
                    direction.dot(&geom.local_x),
                    direction.dot(&geom.local_y),
                    direction.dot(&geom.local_z),
                ) * load.magnitude;
                (local.x, local.y, local.z)
            };

            let (wx, wy, wz) = match load.direction {
                DistributedLoadDirection::LocalX => (load.magnitude, 0.0, 0.0),
                DistributedLoadDirection::LocalY => (0.0, load.magnitude, 0.0),
                DistributedLoadDirection::LocalZ => (0.0, 0.0, load.magnitude),
                DistributedLoadDirection::GlobalX => global(Vector3::x()),
                DistributedLoadDirection::GlobalY => global(Vector3::y()),
                DistributedLoadDirection::GlobalZ => global(Vector3::z()),
            };

            #[rustfmt::skip]
            let fe = Vector12::from_row_slice(&[
                wx*l/2.0, wy*l/2.0, wz*l/2.0,      0.0, -wz*l*l/12.0,  wy*l*l/12.0,
                wx*l/2.0, wy*l/2.0, wz*l/2.0,      0.0,  wz*l*l/12.0, -wy*l*l/12.0,
            ]);

            f_local += fe;
        }

        let f_global = t.transpose() * f_local;
        DVector::from_row_slice(f_global.as_slice())
    }
}

impl Element for Frame3D {
    fn id(&self) -> usize {
        self.id
    }

    fn stiffness_matrix(&self, nodes: &[Node]) -> DMatrix<f64> {
        let k_local = self.local_stiffness_matrix(nodes);
        let t = self.transformation_matrix(nodes);
        let k_global = t.transpose() * k_local * t;

        DMatrix::from_row_slice(12, 12, k_global.as_slice())
    }

    fn dof_indices(&self) -> Vec<usize> {
        vec![
            global_dof_index(self.node_i, Dof::Ux),
            global_dof_index(self.node_i, Dof::Uy),
            global_dof_index(self.node_i, Dof::Uz),
            global_dof_index(self.node_i, Dof::Rx),
            global_dof_index(self.node_i, Dof::Ry),
            global_dof_index(self.node_i, Dof::Rz),
            global_dof_index(self.node_j, Dof::Ux),
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
pub struct Frame3DGeometry {
    pub length: f64,
    pub local_x: Vector3<f64>,
    pub local_y: Vector3<f64>,
    pub local_z: Vector3<f64>,
}
