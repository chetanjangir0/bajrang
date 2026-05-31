use nalgebra::{DMatrix, DVector};

use crate::node::Node;

/// Common FEM element contract for global assembly.
pub trait Element {
    /// Element stiffness matrix in global coordinates.
    fn stiffness_matrix(&self, nodes: &[Node]) -> DMatrix<f64>;

    /// Global DOF indices touched by this element.
    fn dof_indices(&self) -> Vec<usize>;

    /// Equivalent nodal load vector in global coordinates.
    fn equivalent_load_vector(&self, _nodes: &[Node]) -> DVector<f64> {
        DVector::zeros(self.dof_indices().len())
    }
}
