use model::{
    boundary::Support,
    dof::{global_dof_index, DOFS_PER_NODE},
    elements::truss2d::Truss2D,
    load::NodalLoad,
    node::Node,
};
use nalgebra::DMatrix;

use crate::elements::truss2d::stiffness_matrix;

/// Assemble the global stiffness matrix for a 2D truss model.
pub fn assemble_global_stiffness(
    nodes: &[Node],
    elements: &[Truss2D],
) -> DMatrix<f64> {
    let ndof = nodes.len() * DOFS_PER_NODE;
    let mut k_global = DMatrix::<f64>::zeros(ndof, ndof);

    for element in elements {
        let ni = &nodes[element.node_i];
        let nj = &nodes[element.node_j];
        let ke = stiffness_matrix(element, ni, nj);
        let dofs = element.global_dof_indices();

        // Scatter ke into K_global at the correct DOF positions
        for (local_row, &global_row) in dofs.iter().enumerate() {
            for (local_col, &global_col) in dofs.iter().enumerate() {
                k_global[(global_row, global_col)] += ke[(local_row, local_col)];
            }
        }
    }

    k_global
}

/// Assemble the global force vector from nodal loads.
pub fn assemble_load_vector(nodes: &[Node], loads: &[NodalLoad]) -> Vec<f64> {
    let ndof = nodes.len() * DOFS_PER_NODE;
    let mut f = vec![0.0_f64; ndof];

    for load in loads {
        let idx = global_dof_index(load.node_id, load.dof);
        f[idx] += load.magnitude;
    }

    f
}

/// Apply homogeneous boundary conditions using the penalty / elimination method.
///
/// This uses the "zeroing rows and columns" approach:
/// - Zero the constrained row and column
/// - Set the diagonal to 1.0
/// - Zero the corresponding force entry
///
/// Simple, correct, and numerically stable for most structural problems.
pub fn apply_boundary_conditions(
    k: &mut DMatrix<f64>,
    f: &mut Vec<f64>,
    supports: &[Support],
    nodes: &[Node],
) {
    for support in supports {
        let idx = global_dof_index(support.node_id, support.dof);

        // Zero the row and column
        for j in 0..k.ncols() {
            k[(idx, j)] = 0.0;
            k[(j, idx)] = 0.0;
        }

        // Set diagonal to 1 and force to 0
        k[(idx, idx)] = 1.0;
        f[idx] = 0.0;
    }
}
