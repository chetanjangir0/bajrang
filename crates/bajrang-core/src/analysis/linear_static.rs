use model::{
    boundary::Support,
    dof::{Dof, global_dof_index},
    elements::{
        StructuralElement, beam2d::Beam2D, beam3d::Beam3D, frame2d::Frame2D, frame3d::Frame3D,
        traits::Element, truss2d::Truss2D, truss3d::Truss3D,
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
    post::diagrams::{self, MemberDiagram},
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
    pub support_reactions: Vec<SupportReaction>,
    pub member_forces: Vec<f64>,
}

/// Results from a 3D truss analysis.
#[derive(Debug)]
pub struct Truss3DResults {
    pub displacements: Vec<f64>,
    pub support_reactions: Vec<SupportReaction>,
    pub member_forces: Vec<f64>,
}

/// Results from a 2D beam analysis.
#[derive(Debug)]
pub struct Beam2DResults {
    pub displacements: Vec<f64>,
    pub support_reactions: Vec<SupportReaction>,
    pub member_end_forces: Vec<[f64; 4]>,
    pub member_diagrams: Vec<MemberDiagram>,
}

/// Results from a 3D beam analysis.
#[derive(Debug)]
pub struct Beam3DResults {
    pub displacements: Vec<f64>,
    pub support_reactions: Vec<SupportReaction>,
    pub member_end_forces: Vec<[f64; 10]>,
}

/// Results from a 2D frame analysis.
#[derive(Debug)]
pub struct Frame2DResults {
    pub displacements: Vec<f64>,
    pub support_reactions: Vec<SupportReaction>,
    pub member_end_forces: Vec<[f64; 6]>,
    pub member_diagrams: Vec<MemberDiagram>,
}

/// Results from a 3D frame analysis.
#[derive(Debug)]
pub struct Frame3DResults {
    pub displacements: Vec<f64>,
    pub support_reactions: Vec<SupportReaction>,
    pub member_end_forces: Vec<[f64; 12]>,
}

/// Results from a mixed 2D analysis.
#[derive(Debug)]
pub struct Mixed2DResults {
    pub displacements: Vec<f64>,
    pub support_reactions: Vec<SupportReaction>,
    pub member_results: Vec<ElementResult>,
    pub member_diagrams: Vec<MemberDiagram>,
}

/// Reaction at a constrained support DOF.
#[derive(Debug, Clone, PartialEq)]
pub struct SupportReaction {
    pub node_id: usize,
    pub dof: Dof,
    pub magnitude: f64,
}

#[derive(Debug)]
pub enum ElementResult {
    Truss2D { axial_force: f64 },
    Truss3D { axial_force: f64 },
    Beam2D { end_forces: [f64; 4] },
    Beam3D { end_forces: [f64; 10] },
    Frame2D { end_forces: [f64; 6] },
    Frame3D { end_forces: [f64; 12] },
}

pub fn run(
    nodes: &[Node],
    elements: &[Truss2D],
    supports: &[Support],
    loads: &[NodalLoad],
) -> Result<LinearStaticResults, AnalysisError> {
    let (displacements, support_reactions) =
        solve_displacements(nodes, elements, supports, loads, &[])?;

    let member_forces = elements
        .iter()
        .map(|e| e.axial_force(nodes, &displacements))
        .collect();

    Ok(LinearStaticResults {
        displacements,
        support_reactions,
        member_forces,
    })
}

pub fn run_truss3d(
    nodes: &[Node],
    elements: &[Truss3D],
    supports: &[Support],
    loads: &[NodalLoad],
) -> Result<Truss3DResults, AnalysisError> {
    let (displacements, support_reactions) =
        solve_displacements(nodes, elements, supports, loads, &[])?;

    let member_forces = elements
        .iter()
        .map(|element| element.axial_force(nodes, &displacements))
        .collect();

    Ok(Truss3DResults {
        displacements,
        support_reactions,
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
    let (displacements, support_reactions) =
        solve_displacements(nodes, elements, supports, loads, distributed_loads)?;

    let member_end_forces: Vec<[f64; 4]> = elements
        .iter()
        .map(|element| {
            let forces = element.end_forces(nodes, &displacements);
            [forces[0], forces[1], forces[2], forces[3]]
        })
        .collect();
    let member_diagrams =
        diagrams::beam2d_diagrams(nodes, elements, &member_end_forces, distributed_loads, 33);

    Ok(Beam2DResults {
        displacements,
        support_reactions,
        member_end_forces,
        member_diagrams,
    })
}

pub fn run_frame2d(
    nodes: &[Node],
    elements: &[Frame2D],
    supports: &[Support],
    loads: &[NodalLoad],
    distributed_loads: &[DistributedLoad],
) -> Result<Frame2DResults, AnalysisError> {
    let (displacements, support_reactions) =
        solve_displacements(nodes, elements, supports, loads, distributed_loads)?;

    let member_end_forces: Vec<[f64; 6]> = elements
        .iter()
        .map(|element| {
            let forces = element.end_forces(nodes, &displacements);
            [
                forces[0], forces[1], forces[2], forces[3], forces[4], forces[5],
            ]
        })
        .collect();
    let member_diagrams =
        diagrams::frame2d_diagrams(nodes, elements, &member_end_forces, distributed_loads, 33);

    Ok(Frame2DResults {
        displacements,
        support_reactions,
        member_end_forces,
        member_diagrams,
    })
}

pub fn run_beam3d(
    nodes: &[Node],
    elements: &[Beam3D],
    supports: &[Support],
    loads: &[NodalLoad],
    distributed_loads: &[DistributedLoad],
) -> Result<Beam3DResults, AnalysisError> {
    let (displacements, support_reactions) =
        solve_displacements(nodes, elements, supports, loads, distributed_loads)?;

    let member_end_forces = elements
        .iter()
        .map(|element| {
            let forces = element.end_forces(nodes, &displacements);
            [
                forces[0], forces[1], forces[2], forces[3], forces[4], forces[5], forces[6],
                forces[7], forces[8], forces[9],
            ]
        })
        .collect();

    Ok(Beam3DResults {
        displacements,
        support_reactions,
        member_end_forces,
    })
}

pub fn run_frame3d(
    nodes: &[Node],
    elements: &[Frame3D],
    supports: &[Support],
    loads: &[NodalLoad],
    distributed_loads: &[DistributedLoad],
) -> Result<Frame3DResults, AnalysisError> {
    let (displacements, support_reactions) =
        solve_displacements(nodes, elements, supports, loads, distributed_loads)?;

    let member_end_forces = elements
        .iter()
        .map(|element| {
            let forces = element.end_forces(nodes, &displacements);
            [
                forces[0], forces[1], forces[2], forces[3], forces[4], forces[5], forces[6],
                forces[7], forces[8], forces[9], forces[10], forces[11],
            ]
        })
        .collect();

    Ok(Frame3DResults {
        displacements,
        support_reactions,
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
    let (displacements, support_reactions) =
        solve_displacements(nodes, elements, supports, nodal_loads, distributed_loads)?;

    let member_results: Vec<ElementResult> = elements
        .iter()
        .map(|element| match element {
            StructuralElement::Truss2D(truss) => ElementResult::Truss2D {
                axial_force: truss.axial_force(nodes, &displacements),
            },
            StructuralElement::Truss3D(truss) => ElementResult::Truss3D {
                axial_force: truss.axial_force(nodes, &displacements),
            },
            StructuralElement::Beam2D(beam) => {
                let forces = beam.end_forces(nodes, &displacements);
                ElementResult::Beam2D {
                    end_forces: [forces[0], forces[1], forces[2], forces[3]],
                }
            }
            StructuralElement::Beam3D(beam) => {
                let forces = beam.end_forces(nodes, &displacements);
                ElementResult::Beam3D {
                    end_forces: [
                        forces[0], forces[1], forces[2], forces[3], forces[4], forces[5],
                        forces[6], forces[7], forces[8], forces[9],
                    ],
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
            StructuralElement::Frame3D(frame) => {
                let forces = frame.end_forces(nodes, &displacements);
                ElementResult::Frame3D {
                    end_forces: [
                        forces[0], forces[1], forces[2], forces[3], forces[4], forces[5],
                        forces[6], forces[7], forces[8], forces[9], forces[10], forces[11],
                    ],
                }
            }
        })
        .collect();
    let member_diagrams =
        diagrams::mixed_2d_diagrams(nodes, elements, &member_results, distributed_loads, 33);

    Ok(Mixed2DResults {
        displacements,
        support_reactions,
        member_results,
        member_diagrams,
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
) -> Result<(Vec<f64>, Vec<SupportReaction>), AnalysisError> {
    let k = assemble_global_stiffness(nodes, elements);
    let f = assemble_load_vector(nodes, elements, loads, distributed_loads);
    let mut k_constrained = k.clone();
    let mut f_constrained = f.clone();

    let mut constrained_dofs = constrained_dofs_from_supports(supports);
    constrained_dofs.extend(inactive_dofs(nodes, elements));
    constrained_dofs.sort_unstable();
    constrained_dofs.dedup();
    apply_boundary_conditions(&mut k_constrained, &mut f_constrained, &constrained_dofs);

    let displacements = solver::solve(k_constrained, f_constrained).map_err(AnalysisError::from)?;
    let support_reactions = recover_support_reactions(&k, &f, &displacements, supports);

    Ok((displacements, support_reactions))
}

fn recover_support_reactions(
    stiffness: &nalgebra::DMatrix<f64>,
    loads: &[f64],
    displacements: &[f64],
    supports: &[Support],
) -> Vec<SupportReaction> {
    let displacement_vector = nalgebra::DVector::from_column_slice(displacements);
    let applied_load_vector = nalgebra::DVector::from_column_slice(loads);
    let residual = stiffness * displacement_vector - applied_load_vector;

    supports
        .iter()
        .map(|support| {
            let idx = global_dof_index(support.node_id, support.dof);
            SupportReaction {
                node_id: support.node_id,
                dof: support.dof,
                magnitude: residual[idx],
            }
        })
        .collect()
}
