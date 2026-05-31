use model::{
    boundary::Support,
    elements::{
        StructuralElement, beam2d::Beam2D, frame2d::Frame2D, traits::Element, truss2d::Truss2D,
    },
    load::{DistributedLoad, NodalLoad},
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

/// Results from a mixed 2D analysis.
#[derive(Debug)]
pub struct Mixed2DResults {
    pub displacements: Vec<f64>,
    pub member_results: Vec<ElementResult>,
}

#[derive(Debug)]
pub enum ElementResult {
    Truss2D { axial_force: f64 },
    Beam2D { end_forces: [f64; 4] },
    Frame2D { end_forces: [f64; 6] },
}

pub fn run(
    nodes: &[Node],
    elements: &[Truss2D],
    supports: &[Support],
    loads: &[NodalLoad],
) -> Result<LinearStaticResults, AnalysisError> {
    let displacements = solve_displacements(nodes, elements, supports, loads, &[])?;

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
    distributed_loads: &[DistributedLoad],
) -> Result<Beam2DResults, AnalysisError> {
    let displacements = solve_displacements(nodes, elements, supports, loads, distributed_loads)?;

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
    distributed_loads: &[DistributedLoad],
) -> Result<Frame2DResults, AnalysisError> {
    let displacements = solve_displacements(nodes, elements, supports, loads, distributed_loads)?;

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

pub fn run_mixed(
    nodes: &[Node],
    elements: &[StructuralElement],
    supports: &[Support],
    nodal_loads: &[NodalLoad],
    distributed_loads: &[DistributedLoad],
) -> Result<Mixed2DResults, AnalysisError> {
    let displacements =
        solve_displacements(nodes, elements, supports, nodal_loads, distributed_loads)?;

    let member_results = elements
        .iter()
        .map(|element| match element {
            StructuralElement::Truss2D(truss) => ElementResult::Truss2D {
                axial_force: truss.axial_force(nodes, &displacements),
            },
            StructuralElement::Beam2D(beam) => {
                let forces = beam.end_forces(nodes, &displacements);
                ElementResult::Beam2D {
                    end_forces: [forces[0], forces[1], forces[2], forces[3]],
                }
            }
            StructuralElement::Frame2D(frame) => {
                let forces = frame.end_forces(nodes, &displacements);
                ElementResult::Frame2D {
                    end_forces: [
                        forces[0], forces[1], forces[2], forces[3], forces[4], forces[5],
                    ],
                }
            }
        })
        .collect();

    Ok(Mixed2DResults {
        displacements,
        member_results,
    })
}

/// Solve the global displacement field for any element type implementing
/// the shared assembly contract.
pub fn solve_displacements<E: Element>(
    nodes: &[Node],
    elements: &[E],
    supports: &[Support],
    loads: &[NodalLoad],
    distributed_loads: &[DistributedLoad],
) -> Result<Vec<f64>, AnalysisError> {
    // 1. Assemble global system
    let mut k = assemble_global_stiffness(nodes, elements);
    let mut f = assemble_load_vector(nodes, elements, loads, distributed_loads);

    // 2. Apply user supports plus any inactive global DOFs that no element uses.
    let mut constrained_dofs = constrained_dofs_from_supports(supports);
    constrained_dofs.extend(inactive_dofs(nodes, elements));
    constrained_dofs.sort_unstable();
    constrained_dofs.dedup();
    apply_boundary_conditions(&mut k, &mut f, &constrained_dofs);

    // 3. Solve
    solver::solve(k, f).map_err(AnalysisError::from)
}
