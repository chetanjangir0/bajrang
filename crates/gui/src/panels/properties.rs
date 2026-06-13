use iced::widget::{column, container, row, text};
use iced::{Element, Fill};
use model::{dof::Dof, elements::StructuralElement};

use crate::{
    app::Message,
    state::{AnalysisState, Selection, StructuralModel},
    theme,
};

pub fn view<'a>(
    model: &'a StructuralModel,
    selection: Option<Selection>,
    analysis: &'a AnalysisState,
) -> Element<'a, Message> {
    let selected = match selection {
        Some(Selection::Node(id)) => node_properties(model, id),
        Some(Selection::Element(id)) => element_properties(model, id),
        None => empty_selection(),
    };

    column![
        text("Properties").size(18).color(theme::TEXT),
        selected,
        analysis_summary(analysis),
    ]
    .spacing(18)
    .padding(14)
    .width(Fill)
    .into()
}

fn node_properties(model: &StructuralModel, id: usize) -> Element<'_, Message> {
    if let Some(node) = model.nodes.iter().find(|node| node.id == id) {
        column![
            property_header(format!("Node {}", node.id)),
            property_row("X", format!("{:.3} m", node.x)),
            property_row("Y", format!("{:.3} m", node.y)),
            property_row("Z", format!("{:.3} m", node.z)),
            property_row("Supports", support_summary(model, id)),
            property_row("Loads", load_summary(model, id)),
        ]
        .spacing(8)
        .into()
    } else {
        empty_selection()
    }
}

fn element_properties(model: &StructuralModel, id: usize) -> Element<'_, Message> {
    if let Some(element) = model
        .elements
        .iter()
        .find(|element| element_id(element) == id)
    {
        let (kind, node_i, node_j) = element_endpoints(element);

        column![
            property_header(format!("{kind} {id}")),
            property_row("Start node", format!("N{node_i}")),
            property_row("End node", format!("N{node_j}")),
            property_row("Length", format!("{:.3} m", length(model, node_i, node_j))),
        ]
        .spacing(8)
        .into()
    } else {
        empty_selection()
    }
}

fn analysis_summary(analysis: &AnalysisState) -> Element<'_, Message> {
    let content = match analysis {
        AnalysisState::NotRun => column![
            property_header("Analysis"),
            text("No analysis has been run.")
                .size(14)
                .color(theme::TEXT_MUTED),
        ],
        AnalysisState::Success(summary) => column![
            property_header("Analysis"),
            property_row("Scope", summary.result_scope.to_string()),
            property_row("Max |u|", format!("{:.3e} m", summary.max_displacement)),
            property_row("Reactions", summary.reaction_count.to_string()),
        ],
        AnalysisState::Failed(error) => column![
            property_header("Analysis"),
            text(error).size(14).color(theme::DANGER),
        ],
    };

    container(content.spacing(8))
        .padding(10)
        .width(Fill)
        .style(theme::neutral_row)
        .into()
}

fn empty_selection() -> Element<'static, Message> {
    container(
        column![
            property_header("No selection"),
            text("Select a node or member in the viewport.")
                .size(14)
                .color(theme::TEXT_MUTED),
        ]
        .spacing(8),
    )
    .padding(10)
    .width(Fill)
    .style(theme::neutral_row)
    .into()
}

fn property_header(label: impl Into<String>) -> Element<'static, Message> {
    text(label.into()).size(15).color(theme::TEXT).into()
}

fn property_row(label: &'static str, value: String) -> Element<'static, Message> {
    row![
        text(label).size(13).color(theme::TEXT_MUTED).width(110),
        text(value).size(14).color(theme::TEXT),
    ]
    .spacing(8)
    .into()
}

fn support_summary(model: &StructuralModel, node_id: usize) -> String {
    let labels: Vec<&'static str> = model
        .supports
        .iter()
        .filter(|support| support.node_id == node_id)
        .map(|support| dof_label(support.dof))
        .collect();

    if labels.is_empty() {
        "None".to_string()
    } else {
        labels.join(", ")
    }
}

fn load_summary(model: &StructuralModel, node_id: usize) -> String {
    let loads: Vec<String> = model
        .nodal_loads
        .iter()
        .filter(|load| load.node_id == node_id)
        .map(|load| format!("{} {:+.2e}", dof_label(load.dof), load.magnitude))
        .collect();

    if loads.is_empty() {
        "None".to_string()
    } else {
        loads.join(", ")
    }
}

fn length(model: &StructuralModel, node_i: usize, node_j: usize) -> f64 {
    let Some(ni) = model.nodes.iter().find(|node| node.id == node_i) else {
        return 0.0;
    };
    let Some(nj) = model.nodes.iter().find(|node| node.id == node_j) else {
        return 0.0;
    };

    let dx = nj.x - ni.x;
    let dy = nj.y - ni.y;
    let dz = nj.z - ni.z;

    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn element_id(element: &StructuralElement) -> usize {
    match element {
        StructuralElement::Truss2D(element) => element.id,
        StructuralElement::Truss3D(element) => element.id,
        StructuralElement::Beam2D(element) => element.id,
        StructuralElement::Beam3D(element) => element.id,
        StructuralElement::Frame2D(element) => element.id,
        StructuralElement::Frame3D(element) => element.id,
    }
}

fn element_endpoints(element: &StructuralElement) -> (&'static str, usize, usize) {
    match element {
        StructuralElement::Truss2D(element) => ("Truss2D", element.node_i, element.node_j),
        StructuralElement::Truss3D(element) => ("Truss3D", element.node_i, element.node_j),
        StructuralElement::Beam2D(element) => ("Beam2D", element.node_i, element.node_j),
        StructuralElement::Beam3D(element) => ("Beam3D", element.node_i, element.node_j),
        StructuralElement::Frame2D(element) => ("Frame2D", element.node_i, element.node_j),
        StructuralElement::Frame3D(element) => ("Frame3D", element.node_i, element.node_j),
    }
}

fn dof_label(dof: Dof) -> &'static str {
    match dof {
        Dof::Ux => "Ux",
        Dof::Uy => "Uy",
        Dof::Uz => "Uz",
        Dof::Rx => "Rx",
        Dof::Ry => "Ry",
        Dof::Rz => "Rz",
    }
}
