use model::{
    boundary::Support,
    elements::truss2d::Truss2D,
    load::NodalLoad,
    node::Node,
};
use thiserror::Error;

use crate::{
    assembler::{
        apply_boundary_conditions, assemble_global_stiffness, assemble_load_vector,
    },
    elements::truss2d::axial_force,
    solver::{self, SolverError},
};

#[derive(Debug, Error)]
pub enum AnalysisError {
    #[error("Solver failed: {0}")]
    Solver(#[from] SolverError),
}

/// Results from a linear static analysis.
#[derive(Debug)]
pub struct LinearStaticResults {
    /// Global displacement vector (indexed by global DOF)
    pub displacements: Vec<f64>,
    /// Axial force in each truss element (same order as input elements)
    pub member_forces: Vec<f64>,
}

/// Run a linear static analysis on a 2D truss model.
pub fn run(
    nodes: &[Node],
    elements: &[Truss2D],
    supports: &[Support],
    loads: &[NodalLoad],
) -> Result<LinearStaticResults, AnalysisError> {
    // 1. Assemble global system
    let mut k = assemble_global_stiffness(nodes, elements);
    let mut f = assemble_load_vector(nodes, loads);

    // 2. Apply boundary conditions (modifies K and F in place)
    apply_boundary_conditions(&mut k, &mut f, supports, nodes);

    // 3. Solve
    let displacements = solver::solve(k, f)?;

    // 4. Recover member forces
    let member_forces = elements
        .iter()
        .map(|e| {
            let ni = &nodes[e.node_i];
            let nj = &nodes[e.node_j];
            axial_force(e, ni, nj, &displacements)
        })
        .collect();

    Ok(LinearStaticResults {
        displacements,
        member_forces,
    })
}
