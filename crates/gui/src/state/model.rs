use super::{CoordinateAxis, MemberEndpoint, SupportPreset};
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

    pub fn add_default_node(&mut self) -> usize {
        let (x, y) = self
            .nodes
            .last()
            .map_or((0.0, 0.0), |node| (node.x + 1.0, node.y));

        self.add_node(x, y)
    }

    pub fn remove_node(&mut self, node_id: usize) -> Result<(), String> {
        let Some(index) = self.nodes.iter().position(|node| node.id == node_id) else {
            return Err(format!("Node {node_id} does not exist."));
        };

        self.nodes.remove(index);
        self.elements.retain(|element| {
            let (_, node_i, node_j) = element_data(element);
            node_i != node_id && node_j != node_id
        });
        self.supports.retain(|support| support.node_id != node_id);
        self.nodal_loads.retain(|load| load.node_id != node_id);

        Ok(())
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
        if self.node(node_i).is_none() {
            return Err(format!("Node {node_i} does not exist."));
        }

        if self.node(node_j).is_none() {
            return Err(format!("Node {node_j} does not exist."));
        }

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

    pub fn add_default_frame_member(&mut self) -> Result<usize, String> {
        for (index, node_i) in self.nodes.iter().enumerate() {
            for node_j in self.nodes.iter().skip(index + 1) {
                if self.element_between(node_i.id, node_j.id).is_none() {
                    return self.add_frame_member(node_i.id, node_j.id);
                }
            }
        }

        if self.nodes.len() < 2 {
            Err("Add at least two nodes before adding a member.".to_string())
        } else {
            Err("All available node pairs already have members.".to_string())
        }
    }

    pub fn remove_element(&mut self, element_id: usize) -> Result<(), String> {
        let Some(index) = self
            .elements
            .iter()
            .position(|element| element_id_of(element) == element_id)
        else {
            return Err(format!("Member {element_id} does not exist."));
        };

        self.elements.remove(index);
        Ok(())
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

    pub fn add_default_load(&mut self, node_id: usize) -> Result<usize, String> {
        if self.node(node_id).is_none() {
            return Err(format!("Node {node_id} does not exist."));
        }

        self.nodal_loads
            .push(NodalLoad::new(node_id, Dof::Uy, -10_000.0));

        Ok(self.nodal_loads.len() - 1)
    }

    pub fn add_default_load_to_first_node(&mut self) -> Result<usize, String> {
        let Some(node_id) = self.nodes.first().map(|node| node.id) else {
            return Err("Add a node before adding a load.".to_string());
        };

        self.add_default_load(node_id)
    }

    pub fn remove_nodal_load(&mut self, index: usize) -> Result<(), String> {
        if index >= self.nodal_loads.len() {
            return Err(format!("Load {index} does not exist."));
        }

        self.nodal_loads.remove(index);
        Ok(())
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

    pub fn assign_support_preset(
        &mut self,
        node_id: usize,
        preset: SupportPreset,
    ) -> Result<usize, String> {
        let dofs: &[Dof] = match preset {
            SupportPreset::Pin => &[Dof::Ux, Dof::Uy, Dof::Uz],
            SupportPreset::Fixed => &[Dof::Ux, Dof::Uy, Dof::Uz, Dof::Rx, Dof::Ry, Dof::Rz],
            SupportPreset::Roller => &[Dof::Uy],
        };

        self.assign_supports(node_id, dofs, preset.label())
    }

    pub fn assign_custom_support(&mut self, node_id: usize, dofs: &[Dof]) -> Result<usize, String> {
        self.assign_supports(node_id, dofs, "Custom")
    }

    fn assign_supports(
        &mut self,
        node_id: usize,
        dofs: &[Dof],
        label: &str,
    ) -> Result<usize, String> {
        if self.node(node_id).is_none() {
            return Err(format!("Node {node_id} does not exist."));
        }

        if dofs.is_empty() {
            return Err("Select at least one restrained DOF.".to_string());
        }

        let first_added = self.supports.len();

        for dof in dofs {
            let support = Support::new(node_id, *dof);
            if !self
                .supports
                .iter()
                .any(|existing| existing.node_id == support.node_id && existing.dof == support.dof)
            {
                self.supports.push(support);
            }
        }

        if self.supports.len() == first_added {
            Err(format!("{label} support already exists at node {node_id}."))
        } else {
            Ok(first_added)
        }
    }

    pub fn remove_supports_at_node(&mut self, node_id: usize) -> Result<(), String> {
        let initial_len = self.supports.len();
        self.supports.retain(|support| support.node_id != node_id);

        if self.supports.len() == initial_len {
            Err(format!("Node {node_id} has no supports."))
        } else {
            Ok(())
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

    #[test]
    fn pin_support_preset_restrains_all_translations() {
        let mut model = StructuralModel::empty();
        let node_id = model.add_node(0.0, 0.0);

        model
            .assign_support_preset(node_id, SupportPreset::Pin)
            .expect("pin support should be assigned");

        let dofs = support_dofs_at_node(&model, node_id);
        assert_eq!(dofs, vec![Dof::Ux, Dof::Uy, Dof::Uz]);
    }

    #[test]
    fn rejects_custom_support_without_restrained_dofs() {
        let mut model = StructuralModel::empty();
        let node_id = model.add_node(0.0, 0.0);

        let error = model
            .assign_custom_support(node_id, &[])
            .expect_err("empty support should be rejected");

        assert_eq!(error, "Select at least one restrained DOF.");
    }

    #[test]
    fn removes_supports_grouped_by_node() {
        let mut model = StructuralModel::empty();
        let node_id = model.add_node(0.0, 0.0);
        model
            .assign_support_preset(node_id, SupportPreset::Fixed)
            .expect("fixed support should be assigned");

        model
            .remove_supports_at_node(node_id)
            .expect("support group should be removed");

        assert!(model.supports.is_empty());
    }

    #[test]
    fn removing_node_clears_dependent_entities() {
        let mut model = StructuralModel::sample();

        model
            .remove_node(3)
            .expect("node and dependent entities should be removed");

        assert!(model.node(3).is_none());
        assert!(model.elements.iter().all(|element| {
            let (_, node_i, node_j) = element_data(element);
            node_i != 3 && node_j != 3
        }));
        assert!(model.nodal_loads.iter().all(|load| load.node_id != 3));
        assert!(model.supports.iter().all(|support| support.node_id != 3));
    }

    #[test]
    fn adds_default_member_between_available_nodes() {
        let mut model = StructuralModel::empty();
        model.add_node(0.0, 0.0);
        model.add_node(1.0, 0.0);

        let id = model
            .add_default_frame_member()
            .expect("available node pair should create a member");

        assert_eq!(element_data(model.element(id).unwrap()), (id, 0, 1));
    }

    fn support_dofs_at_node(model: &StructuralModel, node_id: usize) -> Vec<Dof> {
        let mut dofs = model
            .supports
            .iter()
            .filter(|support| support.node_id == node_id)
            .map(|support| support.dof)
            .collect::<Vec<_>>();

        dofs.sort_by_key(|dof| match dof {
            Dof::Ux => 0,
            Dof::Uy => 1,
            Dof::Uz => 2,
            Dof::Rx => 3,
            Dof::Ry => 4,
            Dof::Rz => 5,
        });

        dofs
    }
}
