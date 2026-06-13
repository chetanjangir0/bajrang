use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};

use crate::{
    dof::{Dof, global_dof_index},
    elements::traits::Element,
    load::DistributedLoad,
    material::Material,
    node::Node,
    section::Section,
};

/// A 3D truss element connecting two nodes.
///
/// Resists axial load only. Active DOFs:
/// [u_ix, u_iy, u_iz, u_jx, u_jy, u_jz].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Truss3D {
    pub id: usize,
    pub node_i: usize,
    pub node_j: usize,
    pub material: Material,
    pub section: Section,
}

impl Truss3D {
    pub fn new(
        id: usize,
        node_i: usize,
        node_j: usize,
        material: Material,
        section: Section,
    ) -> Self {
        assert_ne!(node_i, node_j, "Element endpoints must be different nodes");
        assert!(section.area > 0.0, "Truss3D requires a positive area");

        Self {
            id,
            node_i,
            node_j,
            material,
            section,
        }
    }

    pub fn geometry(&self, ni: &Node, nj: &Node) -> Truss3DGeometry {
        let dx = nj.x - ni.x;
        let dy = nj.y - ni.y;
        let dz = nj.z - ni.z;
        let length = (dx * dx + dy * dy + dz * dz).sqrt();

        assert!(length > 1e-12, "Element {} has zero length", self.id);

        Truss3DGeometry {
            length,
            l: dx / length,
            m: dy / length,
            n: dz / length,
        }
    }

    pub fn axial_force(&self, nodes: &[Node], displacements: &[f64]) -> f64 {
        let ni = &nodes[self.node_i];
        let nj = &nodes[self.node_j];
        let geom = self.geometry(ni, nj);
        let ae_over_l = self.material.elastic_modulus * self.section.area / geom.length;

        let dofs = self.dof_indices();
        let u_ix = displacements[dofs[0]];
        let u_iy = displacements[dofs[1]];
        let u_iz = displacements[dofs[2]];
        let u_jx = displacements[dofs[3]];
        let u_jy = displacements[dofs[4]];
        let u_jz = displacements[dofs[5]];

        ae_over_l * (geom.l * (u_jx - u_ix) + geom.m * (u_jy - u_iy) + geom.n * (u_jz - u_iz))
    }
}

impl Element for Truss3D {
    fn id(&self) -> usize {
        self.id
    }

    fn stiffness_matrix(&self, nodes: &[Node]) -> DMatrix<f64> {
        let ni = &nodes[self.node_i];
        let nj = &nodes[self.node_j];
        let geom = self.geometry(ni, nj);
        let ae_over_l = self.material.elastic_modulus * self.section.area / geom.length;

        let l = geom.l;
        let m = geom.m;
        let n = geom.n;

        #[rustfmt::skip]
        let k = DMatrix::from_row_slice(6, 6, &[
             l*l,  l*m,  l*n, -l*l, -l*m, -l*n,
             l*m,  m*m,  m*n, -l*m, -m*m, -m*n,
             l*n,  m*n,  n*n, -l*n, -m*n, -n*n,
            -l*l, -l*m, -l*n,  l*l,  l*m,  l*n,
            -l*m, -m*m, -m*n,  l*m,  m*m,  m*n,
            -l*n, -m*n, -n*n,  l*n,  m*n,  n*n,
        ]);

        ae_over_l * k
    }

    fn dof_indices(&self) -> Vec<usize> {
        vec![
            global_dof_index(self.node_i, Dof::Ux),
            global_dof_index(self.node_i, Dof::Uy),
            global_dof_index(self.node_i, Dof::Uz),
            global_dof_index(self.node_j, Dof::Ux),
            global_dof_index(self.node_j, Dof::Uy),
            global_dof_index(self.node_j, Dof::Uz),
        ]
    }

    fn equivalent_load_vector(
        &self,
        _nodes: &[Node],
        _distributed_loads: &[DistributedLoad],
    ) -> DVector<f64> {
        DVector::zeros(6)
    }
}

#[derive(Debug, Clone)]
pub struct Truss3DGeometry {
    pub length: f64,
    pub l: f64,
    pub m: f64,
    pub n: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{material::Material, node::Node, section::Section};

    fn assert_close(actual: f64, expected: f64, tol: f64, label: &str) {
        assert!(
            (actual - expected).abs() <= tol,
            "{label}: expected {expected:.12e}, got {actual:.12e}"
        );
    }

    #[test]
    fn geometry_for_two_three_six_member_is_normalized() {
        let truss = Truss3D::new(0, 0, 1, Material::new(200.0e9, 0.3), Section::truss(0.01));
        let ni = Node::new_3d(0, 0.0, 0.0, 0.0);
        let nj = Node::new_3d(1, 2.0, 3.0, 6.0);

        let geom = truss.geometry(&ni, &nj);

        assert_close(geom.length, 7.0, 1e-12, "Length");
        assert_close(geom.l, 2.0 / 7.0, 1e-12, "X direction cosine");
        assert_close(geom.m, 3.0 / 7.0, 1e-12, "Y direction cosine");
        assert_close(geom.n, 6.0 / 7.0, 1e-12, "Z direction cosine");
    }
}
