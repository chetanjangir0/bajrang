use model::{
    boundary::Support,
    elements::{beam2d::Beam2D, frame2d::Frame2D, traits::Element, truss2d::Truss2D},
    load::NodalLoad,
    node::Node,
};
use thiserror::Error;

use crate::{
    assembler::{
        apply_boundary_conditions, assemble_global_stiffness, assemble_load_vector,
        boundary_conditions::constrained_dofs_from_supports, inactive_dofs,
    },
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
    pub displacements: Vec<f64>,
    pub member_forces: Vec<f64>,
}

/// Results from a 2D beam analysis.
#[derive(Debug)]
pub struct Beam2DResults {
    pub displacements: Vec<f64>,
    pub member_end_forces: Vec<[f64; 4]>,
}

/// Results from a 2D frame analysis.
#[derive(Debug)]
pub struct Frame2DResults {
    pub displacements: Vec<f64>,
    pub member_end_forces: Vec<[f64; 6]>,
}

pub fn run(
    nodes: &[Node],
    elements: &[Truss2D],
    supports: &[Support],
    loads: &[NodalLoad],
) -> Result<LinearStaticResults, AnalysisError> {
    let displacements = solve_displacements(nodes, elements, supports, loads)?;

    let member_forces = elements
        .iter()
        .map(|e| e.axial_force(nodes, &displacements))
        .collect();

    Ok(LinearStaticResults {
        displacements,
        member_forces,
    })
}

pub fn run_beam2d(
    nodes: &[Node],
    elements: &[Beam2D],
    supports: &[Support],
    loads: &[NodalLoad],
) -> Result<Beam2DResults, AnalysisError> {
    let displacements = solve_displacements(nodes, elements, supports, loads)?;

    let member_end_forces = elements
        .iter()
        .map(|element| {
            let forces = element.end_forces(nodes, &displacements);
            [forces[0], forces[1], forces[2], forces[3]]
        })
        .collect();

    Ok(Beam2DResults {
        displacements,
        member_end_forces,
    })
}

pub fn run_frame2d(
    nodes: &[Node],
    elements: &[Frame2D],
    supports: &[Support],
    loads: &[NodalLoad],
) -> Result<Frame2DResults, AnalysisError> {
    let displacements = solve_displacements(nodes, elements, supports, loads)?;

    let member_end_forces = elements
        .iter()
        .map(|element| {
            let forces = element.end_forces(nodes, &displacements);
            [
                forces[0], forces[1], forces[2], forces[3], forces[4], forces[5],
            ]
        })
        .collect();

    Ok(Frame2DResults {
        displacements,
        member_end_forces,
    })
}

/// Solve the global displacement field for any element type implementing
/// the shared assembly contract.
pub fn solve_displacements<E: Element>(
    nodes: &[Node],
    elements: &[E],
    supports: &[Support],
    loads: &[NodalLoad],
) -> Result<Vec<f64>, AnalysisError> {
    // 1. Assemble global system
    let mut k = assemble_global_stiffness(nodes, elements);
    let mut f = assemble_load_vector(nodes, elements, loads);

    // 2. Apply user supports plus any inactive global DOFs that no element uses.
    let mut constrained_dofs = constrained_dofs_from_supports(supports);
    constrained_dofs.extend(inactive_dofs(nodes, elements));
    constrained_dofs.sort_unstable();
    constrained_dofs.dedup();
    apply_boundary_conditions(&mut k, &mut f, &constrained_dofs);

    // 3. Solve
    solver::solve(k, f).map_err(AnalysisError::from)
}
