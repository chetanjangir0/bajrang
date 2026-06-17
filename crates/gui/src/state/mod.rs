use bajrang_core::analysis::linear_static::{self, ElementResult, SupportReaction};
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
        let frame_section = Section::new(0.006, 8.0e-6);
        let truss_section = Section::truss(0.004);

        let nodes = vec![
            Node::new(0, 0.0, 0.0),
            Node::new(1, 4.0, 0.0),
            Node::new(2, 8.0, 0.0),
            Node::new(3, 4.0, 3.0),
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
            name: "Mixed 2D frame study".to_string(),
            nodes,
            elements,
            supports,
            nodal_loads: vec![NodalLoad::new(3, Dof::Uy, -20_000.0)],
            distributed_loads: Vec::new(),
        }
    }

    pub fn empty() -> Self {
        Self {
            name: "Untitled model".to_string(),
            nodes: Vec::new(),
            elements: Vec::new(),
            supports: Vec::new(),
            nodal_loads: Vec::new(),
            distributed_loads: Vec::new(),
        }
    }

    pub fn add_node(&mut self, x: f64, y: f64) -> usize {
        let id = self.next_node_id();
        self.nodes.push(Node::new(id, snapped(x), snapped(y)));
        id
    }

    pub fn update_node_coordinate(
        &mut self,
        node_id: usize,
        axis: CoordinateAxis,
        value: f64,
    ) -> Result<(), String> {
        let Some(node) = self.nodes.iter_mut().find(|node| node.id == node_id) else {
            return Err(format!("Node {node_id} does not exist."));
        };

        match axis {
            CoordinateAxis::X => node.x = value,
            CoordinateAxis::Y => node.y = value,
            CoordinateAxis::Z => node.z = value,
        }

        Ok(())
    }

    pub fn add_frame_member(&mut self, node_i: usize, node_j: usize) -> Result<usize, String> {
        if node_i == node_j {
            return Err("Member endpoints must be different nodes.".to_string());
        }

        if self.element_between(node_i, node_j).is_some() {
            return Err("A member already connects those nodes.".to_string());
        }

        let id = self.next_element_id();
        self.elements.push(StructuralElement::Frame2D(Frame2D::new(
            id,
            node_i,
            node_j,
            Material::steel(),
            Section::new(0.006, 8.0e-6),
        )));

        Ok(id)
    }

    pub fn update_member_endpoint(
        &mut self,
        element_id: usize,
        endpoint: MemberEndpoint,
        node_id: usize,
    ) -> Result<(), String> {
        if self.node(node_id).is_none() {
            return Err(format!("Node {node_id} does not exist."));
        }

        let Some((current_i, current_j)) = self.element(element_id).map(element_nodes) else {
            return Err(format!("Member {element_id} does not exist."));
        };

        let (node_i, node_j) = match endpoint {
            MemberEndpoint::Start => (node_id, current_j),
            MemberEndpoint::End => (current_i, node_id),
        };

        if node_i == node_j {
            return Err("Member endpoints must be different nodes.".to_string());
        }

        if self
            .elements
            .iter()
            .filter(|element| element_id_of(element) != element_id)
            .any(|element| {
                let (other_i, other_j) = element_nodes(element);
                (other_i == node_i && other_j == node_j) || (other_i == node_j && other_j == node_i)
            })
        {
            return Err("A member already connects those nodes.".to_string());
        }

        let Some(element) = self
            .elements
            .iter_mut()
            .find(|element| element_id_of(element) == element_id)
        else {
            return Err(format!("Member {element_id} does not exist."));
        };

        set_element_nodes(element, node_i, node_j);
        Ok(())
    }

    pub fn add_default_load(&mut self, node_id: usize) {
        self.nodal_loads
            .push(NodalLoad::new(node_id, Dof::Uy, -10_000.0));
    }

    pub fn update_nodal_load(
        &mut self,
        index: usize,
        node_id: usize,
        dof: Dof,
        magnitude: f64,
    ) -> Result<(), String> {
        if self.node(node_id).is_none() {
            return Err(format!("Node {node_id} does not exist."));
        }

        let Some(load) = self.nodal_loads.get_mut(index) else {
            return Err(format!("Load {index} does not exist."));
        };

        load.node_id = node_id;
        load.dof = dof;
        load.magnitude = magnitude;
        Ok(())
    }

    pub fn assign_pin_support(&mut self, node_id: usize) {
        for support in Support::pin(node_id) {
            if !self
                .supports
                .iter()
                .any(|existing| existing.node_id == support.node_id && existing.dof == support.dof)
            {
                self.supports.push(support);
            }
        }
    }

    pub fn update_support(&mut self, index: usize, node_id: usize, dof: Dof) -> Result<(), String> {
        if self.node(node_id).is_none() {
            return Err(format!("Node {node_id} does not exist."));
        }

        if self
            .supports
            .iter()
            .enumerate()
            .any(|(existing_index, support)| {
                existing_index != index && support.node_id == node_id && support.dof == dof
            })
        {
            return Err(format!(
                "Support {node_id} {} already exists.",
                dof_label(dof)
            ));
        }

        let Some(support) = self.supports.get_mut(index) else {
            return Err(format!("Support {index} does not exist."));
        };

        support.node_id = node_id;
        support.dof = dof;
        Ok(())
    }

    pub fn node(&self, id: usize) -> Option<&Node> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn element(&self, id: usize) -> Option<&StructuralElement> {
        self.elements
            .iter()
            .find(|element| element_id(element) == id)
    }

    fn next_node_id(&self) -> usize {
        self.nodes
            .iter()
            .map(|node| node.id)
            .max()
            .map_or(0, |id| id + 1)
    }

    fn next_element_id(&self) -> usize {
        self.elements
            .iter()
            .map(element_id)
            .max()
            .map_or(0, |id| id + 1)
    }

    fn element_between(&self, node_i: usize, node_j: usize) -> Option<usize> {
        self.elements.iter().find_map(|element| {
            let (id, a, b) = element_data(element);
            ((a == node_i && b == node_j) || (a == node_j && b == node_i)).then_some(id)
        })
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

    pub fn marker(self) -> &'static str {
        match self {
            Self::Select => "S",
            Self::AddNode => "+",
            Self::DrawMember => "M",
            Self::AssignLoad => "F",
            Self::AssignSupport => "P",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoordinateAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemberEndpoint {
    Start,
    End,
}

impl MemberEndpoint {
    pub fn label(self) -> &'static str {
        match self {
            Self::Start => "i",
            Self::End => "j",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoadField {
    Node,
    Dof,
    Magnitude,
}

impl LoadField {
    pub fn label(self) -> &'static str {
        match self {
            Self::Node => "N",
            Self::Dof => "DOF",
            Self::Magnitude => "kN",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SupportField {
    Node,
    Dof,
}

impl SupportField {
    pub fn label(self) -> &'static str {
        match self {
            Self::Node => "N",
            Self::Dof => "DOF",
        }
    }
}

impl CoordinateAxis {
    pub fn label(self) -> &'static str {
        match self {
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
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
            Self::Element(id) => format!("Member {id}"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct InteractionDraft {
    pub member_start: Option<usize>,
}

impl InteractionDraft {
    pub fn clear(&mut self) {
        self.member_start = None;
    }
}

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
    pub max_reaction: f64,
    pub max_member_force: f64,
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
    Combined,
}

impl ResultDisplay {
    pub const ALL: [Self; 6] = [
        Self::Model,
        Self::Deformed,
        Self::Displacements,
        Self::Reactions,
        Self::MemberForces,
        Self::Combined,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Model => "Model",
            Self::Deformed => "Deformed",
            Self::Displacements => "Displacements",
            Self::Reactions => "Reactions",
            Self::MemberForces => "Forces",
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
    })
}

pub fn element_id(element: &StructuralElement) -> usize {
    element_data(element).0
}

pub fn element_data(element: &StructuralElement) -> (usize, usize, usize) {
    match element {
        StructuralElement::Truss2D(element) => (element.id, element.node_i, element.node_j),
        StructuralElement::Truss3D(element) => (element.id, element.node_i, element.node_j),
        StructuralElement::Beam2D(element) => (element.id, element.node_i, element.node_j),
        StructuralElement::Beam3D(element) => (element.id, element.node_i, element.node_j),
        StructuralElement::Frame2D(element) => (element.id, element.node_i, element.node_j),
        StructuralElement::Frame3D(element) => (element.id, element.node_i, element.node_j),
    }
}

fn element_id_of(element: &StructuralElement) -> usize {
    element_id(element)
}

fn element_nodes(element: &StructuralElement) -> (usize, usize) {
    let (_, node_i, node_j) = element_data(element);
    (node_i, node_j)
}

fn set_element_nodes(element: &mut StructuralElement, node_i: usize, node_j: usize) {
    match element {
        StructuralElement::Truss2D(element) => {
            element.node_i = node_i;
            element.node_j = node_j;
        }
        StructuralElement::Truss3D(element) => {
            element.node_i = node_i;
            element.node_j = node_j;
        }
        StructuralElement::Beam2D(element) => {
            element.node_i = node_i;
            element.node_j = node_j;
        }
        StructuralElement::Beam3D(element) => {
            element.node_i = node_i;
            element.node_j = node_j;
        }
        StructuralElement::Frame2D(element) => {
            element.node_i = node_i;
            element.node_j = node_j;
        }
        StructuralElement::Frame3D(element) => {
            element.node_i = node_i;
            element.node_j = node_j;
        }
    }
}

pub fn element_kind(element: &StructuralElement) -> &'static str {
    match element {
        StructuralElement::Truss2D(_) => "Truss2D",
        StructuralElement::Truss3D(_) => "Truss3D",
        StructuralElement::Beam2D(_) => "Beam2D",
        StructuralElement::Beam3D(_) => "Beam3D",
        StructuralElement::Frame2D(_) => "Frame2D",
        StructuralElement::Frame3D(_) => "Frame3D",
    }
}

pub fn dof_label(dof: Dof) -> &'static str {
    match dof {
        Dof::Ux => "Ux",
        Dof::Uy => "Uy",
        Dof::Uz => "Uz",
        Dof::Rx => "Rx",
        Dof::Ry => "Ry",
        Dof::Rz => "Rz",
    }
}

pub fn member_length(model: &StructuralModel, node_i: usize, node_j: usize) -> Option<f64> {
    let ni = model.node(node_i)?;
    let nj = model.node(node_j)?;
    let dx = nj.x - ni.x;
    let dy = nj.y - ni.y;
    let dz = nj.z - ni.z;

    Some((dx * dx + dy * dy + dz * dz).sqrt())
}

fn max_abs(values: &[f64]) -> f64 {
    values
        .iter()
        .fold(0.0, |max, value| f64::max(max, value.abs()))
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

fn snapped(value: f64) -> f64 {
    (value * 4.0).round() / 4.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updates_member_endpoint_when_node_exists() {
        let mut model = StructuralModel::sample();

        model
            .update_member_endpoint(1, MemberEndpoint::End, 3)
            .expect("member endpoint should update");

        assert_eq!(element_data(model.element(1).unwrap()), (1, 1, 3));
    }

    #[test]
    fn rejects_duplicate_member_connection() {
        let mut model = StructuralModel::sample();

        let error = model
            .update_member_endpoint(0, MemberEndpoint::End, 3)
            .expect_err("duplicate member should be rejected");

        assert_eq!(error, "A member already connects those nodes.");
    }

    #[test]
    fn updates_load_fields() {
        let mut model = StructuralModel::sample();

        model
            .update_nodal_load(0, 1, Dof::Ux, 12_000.0)
            .expect("load should update");

        let load = &model.nodal_loads[0];
        assert_eq!(load.node_id, 1);
        assert_eq!(load.dof, Dof::Ux);
        assert_eq!(load.magnitude, 12_000.0);
    }

    #[test]
    fn rejects_duplicate_support_constraint() {
        let mut model = StructuralModel::sample();

        let error = model
            .update_support(1, 0, Dof::Ux)
            .expect_err("duplicate support should be rejected");

        assert_eq!(error, "Support 0 Ux already exists.");
    }
}
