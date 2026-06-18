use super::{StructuralModel, element_id, element_kind};
use bajrang_core::analysis::linear_static::{self, ElementResult, SupportReaction};
use bajrang_core::post::diagrams::{DiagramKind, MemberDiagram};
use model::elements::StructuralElement;

#[derive(Debug, Clone)]
pub enum AnalysisState {
    Idle,
    Success(AnalysisSummary),
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct AnalysisSummary {
    pub max_displacement: f64,
    pub reaction_count: usize,
    pub result_scope: &'static str,
    pub displacements: Vec<f64>,
    pub reactions: Vec<SupportReaction>,
    pub member_results: Vec<MemberResultSummary>,
    pub member_diagrams: Vec<MemberDiagram>,
    pub max_reaction: f64,
    pub max_member_force: f64,
    pub max_shear: f64,
    pub max_moment: f64,
}

#[derive(Debug, Clone)]
pub struct MemberResultSummary {
    pub element_id: usize,
    pub kind: &'static str,
    pub values: Vec<(&'static str, f64)>,
    pub governing_force: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultDisplay {
    Model,
    Deformed,
    Displacements,
    Reactions,
    MemberForces,
    ShearForce,
    BendingMoment,
    Combined,
}

impl ResultDisplay {
    pub const ALL: [Self; 8] = [
        Self::Model,
        Self::Deformed,
        Self::Displacements,
        Self::Reactions,
        Self::MemberForces,
        Self::ShearForce,
        Self::BendingMoment,
        Self::Combined,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Model => "Model",
            Self::Deformed => "Deformed",
            Self::Displacements => "Displacements",
            Self::Reactions => "Reactions",
            Self::MemberForces => "Forces",
            Self::ShearForce => "Shear",
            Self::BendingMoment => "Moment",
            Self::Combined => "Combined",
        }
    }

    pub fn needs_results(self) -> bool {
        !matches!(self, Self::Model)
    }
}

pub fn run_basic_analysis(model: &StructuralModel) -> Result<AnalysisSummary, String> {
    if model.elements.is_empty() {
        return Err("Add at least one supported member before solving.".to_string());
    }

    let results = linear_static::run_mixed(
        &model.nodes,
        &model.elements,
        &model.supports,
        &model.nodal_loads,
        &model.distributed_loads,
    )
    .map_err(|error| error.to_string())?;

    let member_results = model
        .elements
        .iter()
        .zip(results.member_results.iter())
        .map(|(element, result)| member_result_summary(element, result))
        .collect::<Vec<_>>();
    let max_shear = max_diagram_abs(&results.member_diagrams, DiagramSelector::Shear);
    let max_moment = max_diagram_abs(&results.member_diagrams, DiagramSelector::Moment);

    Ok(AnalysisSummary {
        max_displacement: max_abs(&results.displacements),
        reaction_count: results.support_reactions.len(),
        result_scope: "Mixed members",
        max_reaction: results
            .support_reactions
            .iter()
            .fold(0.0_f64, |max, reaction| max.max(reaction.magnitude.abs())),
        max_member_force: member_results
            .iter()
            .fold(0.0_f64, |max, result| max.max(result.governing_force.abs())),
        displacements: results.displacements,
        reactions: results.support_reactions,
        member_results,
        member_diagrams: results.member_diagrams,
        max_shear,
        max_moment,
    })
}

fn max_abs(values: &[f64]) -> f64 {
    values
        .iter()
        .fold(0.0, |max, value| f64::max(max, value.abs()))
}

enum DiagramSelector {
    Shear,
    Moment,
}

fn max_diagram_abs(diagrams: &[MemberDiagram], selector: DiagramSelector) -> f64 {
    diagrams
        .iter()
        .filter(|diagram| {
            matches!(
                (&selector, diagram.kind),
                (DiagramSelector::Shear, DiagramKind::ShearY)
                    | (DiagramSelector::Moment, DiagramKind::MomentZ)
            )
        })
        .fold(0.0, |max, diagram| f64::max(max, diagram.max_abs_value()))
}

fn member_result_summary(
    element: &StructuralElement,
    result: &ElementResult,
) -> MemberResultSummary {
    let element_id = element_id(element);
    let kind = element_kind(element);

    match result {
        ElementResult::Truss2D { axial_force } | ElementResult::Truss3D { axial_force } => {
            MemberResultSummary {
                element_id,
                kind,
                values: vec![("Axial", *axial_force)],
                governing_force: *axial_force,
            }
        }
        ElementResult::Beam2D { end_forces } => end_force_summary(
            element_id,
            kind,
            &[
                ("Vi", end_forces[0]),
                ("Mi", end_forces[1]),
                ("Vj", end_forces[2]),
                ("Mj", end_forces[3]),
            ],
        ),
        ElementResult::Beam3D { end_forces } => end_force_summary(
            element_id,
            kind,
            &[
                ("Vy i", end_forces[0]),
                ("Vz i", end_forces[1]),
                ("T i", end_forces[2]),
                ("My i", end_forces[3]),
                ("Mz i", end_forces[4]),
                ("Vy j", end_forces[5]),
                ("Vz j", end_forces[6]),
                ("T j", end_forces[7]),
                ("My j", end_forces[8]),
                ("Mz j", end_forces[9]),
            ],
        ),
        ElementResult::Frame2D { end_forces } => end_force_summary(
            element_id,
            kind,
            &[
                ("Ni", end_forces[0]),
                ("Vi", end_forces[1]),
                ("Mi", end_forces[2]),
                ("Nj", end_forces[3]),
                ("Vj", end_forces[4]),
                ("Mj", end_forces[5]),
            ],
        ),
        ElementResult::Frame3D { end_forces } => end_force_summary(
            element_id,
            kind,
            &[
                ("N i", end_forces[0]),
                ("Vy i", end_forces[1]),
                ("Vz i", end_forces[2]),
                ("T i", end_forces[3]),
                ("My i", end_forces[4]),
                ("Mz i", end_forces[5]),
                ("N j", end_forces[6]),
                ("Vy j", end_forces[7]),
                ("Vz j", end_forces[8]),
                ("T j", end_forces[9]),
                ("My j", end_forces[10]),
                ("Mz j", end_forces[11]),
            ],
        ),
    }
}

fn end_force_summary(
    element_id: usize,
    kind: &'static str,
    values: &[(&'static str, f64)],
) -> MemberResultSummary {
    MemberResultSummary {
        element_id,
        kind,
        values: values.to_vec(),
        governing_force: values
            .iter()
            .fold(0.0_f64, |max, (_, value)| max.max(value.abs())),
    }
}
