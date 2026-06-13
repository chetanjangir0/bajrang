use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};

use crate::{
    elements::{
        beam::{beam2d::Beam2D, beam3d::Beam3D},
        frame::{frame2d::Frame2D, frame3d::Frame3D},
        traits::Element,
        truss::truss2d::Truss2D,
        truss::truss3d::Truss3D,
    },
    load::DistributedLoad,
    node::Node,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StructuralElement {
    Truss2D(Truss2D),
    Truss3D(Truss3D),
    Beam2D(Beam2D),
    Beam3D(Beam3D),
    Frame2D(Frame2D),
    Frame3D(Frame3D),
}

impl Element for StructuralElement {
    fn id(&self) -> usize {
        match self {
            Self::Truss2D(element) => element.id(),
            Self::Truss3D(element) => element.id(),
            Self::Beam2D(element) => element.id(),
            Self::Beam3D(element) => element.id(),
            Self::Frame2D(element) => element.id(),
            Self::Frame3D(element) => element.id(),
        }
    }

    fn stiffness_matrix(&self, nodes: &[Node]) -> DMatrix<f64> {
        match self {
            Self::Truss2D(element) => element.stiffness_matrix(nodes),
            Self::Truss3D(element) => element.stiffness_matrix(nodes),
            Self::Beam2D(element) => element.stiffness_matrix(nodes),
            Self::Beam3D(element) => element.stiffness_matrix(nodes),
            Self::Frame2D(element) => element.stiffness_matrix(nodes),
            Self::Frame3D(element) => element.stiffness_matrix(nodes),
        }
    }

    fn dof_indices(&self) -> Vec<usize> {
        match self {
            Self::Truss2D(element) => element.dof_indices(),
            Self::Truss3D(element) => element.dof_indices(),
            Self::Beam2D(element) => element.dof_indices(),
            Self::Beam3D(element) => element.dof_indices(),
            Self::Frame2D(element) => element.dof_indices(),
            Self::Frame3D(element) => element.dof_indices(),
        }
    }

    fn equivalent_load_vector(
        &self,
        nodes: &[Node],
        distributed_loads: &[DistributedLoad],
    ) -> DVector<f64> {
        match self {
            Self::Truss2D(element) => element.equivalent_load_vector(nodes, distributed_loads),
            Self::Truss3D(element) => element.equivalent_load_vector(nodes, distributed_loads),
            Self::Beam2D(element) => element.equivalent_load_vector(nodes, distributed_loads),
            Self::Beam3D(element) => element.equivalent_load_vector(nodes, distributed_loads),
            Self::Frame2D(element) => element.equivalent_load_vector(nodes, distributed_loads),
            Self::Frame3D(element) => element.equivalent_load_vector(nodes, distributed_loads),
        }
    }
}
