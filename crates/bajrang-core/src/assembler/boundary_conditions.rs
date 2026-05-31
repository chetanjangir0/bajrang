use std::collections::HashSet;

use model::{boundary::Support, elements::traits::Element, node::Node};
use nalgebra::DMatrix;

/// DOFs not touched by any element stiffness contribution.
pub fn inactive_dofs<E: Element>(nodes: &[Node], elements: &[E]) -> Vec<usize> {
    let ndof = nodes.len() * model::dof::DOFS_PER_NODE;
    let active: HashSet<usize> = elements
        .iter()
        .flat_map(|element| element.dof_indices().into_iter())
        .collect();

    (0..ndof).filter(|idx| !active.contains(idx)).collect()
}

/// Apply homogeneous boundary conditions using the elimination method.
pub fn apply_boundary_conditions(
    k: &mut DMatrix<f64>,
    f: &mut Vec<f64>,
    constrained_dofs: &[usize],
) {
    for &idx in constrained_dofs {
        for j in 0..k.ncols() {
            k[(idx, j)] = 0.0;
            k[(j, idx)] = 0.0;
        }

        k[(idx, idx)] = 1.0;
        f[idx] = 0.0;
    }
}

pub fn constrained_dofs_from_supports(supports: &[Support]) -> Vec<usize> {
    supports
        .iter()
        .map(|support| model::dof::global_dof_index(support.node_id, support.dof))
        .collect()
}
