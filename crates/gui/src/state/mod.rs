use bajrang_core::analysis::linear_static;
use model::{
    boundary::Support,
    dof::Dof,
    elements::{StructuralElement, frame2d::Frame2D, truss2d::Truss2D},
    load::{DistributedLoad, NodalLoad},
    material::Material,
    node::Node,
    section::Section,
};

#[derive(Debug, Clone)]
pub struct StructuralModel {
    pub name: String,
    pub nodes: Vec<Node>,
    pub elements: Vec<StructuralElement>,
    pub supports: Vec<Support>,
    pub nodal_loads: Vec<NodalLoad>,
    pub distributed_loads: Vec<DistributedLoad>,
}

impl StructuralModel {
    pub fn sample() -> Self {
        let material = Material::steel();
        let truss_section = Section::truss(0.004);
        let frame_section = Section::new(0.006, 8.0e-6);

        let nodes = vec![
            Node::new(0, 0.0, 0.0),
            Node::new(1, 5.0, 0.0),
            Node::new(2, 10.0, 0.0),
            Node::new(3, 5.0, 3.0),
        ];

        let elements = vec![
            StructuralElement::Frame2D(Frame2D::new(
                0,
                0,
                1,
                material.clone(),
                frame_section.clone(),
            )),
            StructuralElement::Frame2D(Frame2D::new(1, 1, 2, material.clone(), frame_section)),
            StructuralElement::Truss2D(Truss2D::new(
                2,
                0,
                3,
                material.clone(),
                truss_section.clone(),
            )),
            StructuralElement::Truss2D(Truss2D::new(3, 3, 2, material, truss_section)),
        ];

        let mut supports = Support::pin(0);
        supports.extend(Support::roller_y(2));

        Self {
            name: "Untitled 2D mixed frame".to_string(),
            nodes,
            elements,
            supports,
            nodal_loads: vec![NodalLoad::new(3, Dof::Uy, -20_000.0)],
            distributed_loads: Vec::new(),
        }
    }

    pub fn truss2d_elements(&self) -> Vec<Truss2D> {
        self.elements
            .iter()
            .filter_map(|element| match element {
                StructuralElement::Truss2D(element) => Some(element.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn frame2d_elements(&self) -> Vec<Frame2D> {
        self.elements
            .iter()
            .filter_map(|element| match element {
                StructuralElement::Frame2D(element) => Some(element.clone()),
                _ => None,
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceTool {
    Select,
    AddNode,
    DrawMember,
    AssignLoad,
    AssignSupport,
}

impl WorkspaceTool {
    pub const ALL: [Self; 5] = [
        Self::Select,
        Self::AddNode,
        Self::DrawMember,
        Self::AssignLoad,
        Self::AssignSupport,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::AddNode => "Node",
            Self::DrawMember => "Member",
            Self::AssignLoad => "Load",
            Self::AssignSupport => "Support",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    Node(usize),
    Element(usize),
}

impl Selection {
    pub fn label(self) -> String {
        match self {
            Self::Node(id) => format!("Node {id}"),
            Self::Element(id) => format!("Element {id}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AnalysisState {
    NotRun,
    Success(AnalysisSummary),
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct AnalysisSummary {
    pub max_displacement: f64,
    pub reaction_count: usize,
    pub result_scope: &'static str,
}

pub fn run_basic_analysis(model: &StructuralModel) -> Result<AnalysisSummary, String> {
    let frames = model.frame2d_elements();

    if !frames.is_empty() {
        let results = linear_static::run_frame2d(
            &model.nodes,
            &frames,
            &model.supports,
            &model.nodal_loads,
            &model.distributed_loads,
        )
        .map_err(|error| error.to_string())?;

        return Ok(AnalysisSummary {
            max_displacement: max_abs(&results.displacements),
            reaction_count: results.support_reactions.len(),
            result_scope: "2D frame subset",
        });
    }

    let trusses = model.truss2d_elements();

    if !trusses.is_empty() {
        let results =
            linear_static::run(&model.nodes, &trusses, &model.supports, &model.nodal_loads)
                .map_err(|error| error.to_string())?;

        return Ok(AnalysisSummary {
            max_displacement: max_abs(&results.displacements),
            reaction_count: results.support_reactions.len(),
            result_scope: "2D truss",
        });
    }

    Err("Add at least one supported 2D truss or frame element before solving.".to_string())
}

fn max_abs(values: &[f64]) -> f64 {
    values
        .iter()
        .fold(0.0, |max, value| f64::max(max, value.abs()))
}
