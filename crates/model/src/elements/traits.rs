use nalgebra::{DMatrix, DVector};

use crate::{load::DistributedLoad, node::Node};

/// Common FEM element contract for global assembly.
pub trait Element {
    /// Stable element identifier used by external model data such as member loads.
    fn id(&self) -> usize;

    /// Element stiffness matrix in global coordinates.
    fn stiffness_matrix(&self, nodes: &[Node]) -> DMatrix<f64>;

    /// Global DOF indices touched by this element.
    fn dof_indices(&self) -> Vec<usize>;

    /// Equivalent nodal load vector in global coordinates.
    fn equivalent_load_vector(
        &self,
        _nodes: &[Node],
        _distributed_loads: &[DistributedLoad],
    ) -> DVector<f64> {
        DVector::zeros(self.dof_indices().len())
    }
}
