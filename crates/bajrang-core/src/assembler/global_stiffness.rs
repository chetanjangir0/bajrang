use model::{dof::DOFS_PER_NODE, elements::traits::Element, node::Node};
use nalgebra::DMatrix;

/// Assemble the global stiffness matrix for a structural model.
pub fn assemble_global_stiffness<E: Element>(nodes: &[Node], elements: &[E]) -> DMatrix<f64> {
    let ndof = nodes.len() * DOFS_PER_NODE;
    let mut k_global = DMatrix::<f64>::zeros(ndof, ndof);

    for element in elements {
        let ke = element.stiffness_matrix(nodes);
        let dofs = element.dof_indices();

        assert_eq!(
            ke.nrows(),
            dofs.len(),
            "Element stiffness matrix row count must match DOF count",
        );
        assert_eq!(
            ke.ncols(),
            dofs.len(),
            "Element stiffness matrix column count must match DOF count",
        );

        for (local_row, &global_row) in dofs.iter().enumerate() {
            for (local_col, &global_col) in dofs.iter().enumerate() {
                k_global[(global_row, global_col)] += ke[(local_row, local_col)];
            }
        }
    }

    k_global
}
