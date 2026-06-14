use iced::widget::{column, container, row, text};
use iced::{Element, Fill, Length};

use crate::{
    app::Message,
    state::{
        AnalysisState, Selection, StructuralModel, dof_label, element_data, element_kind,
        member_length,
    },
    theme,
};

pub fn view<'a>(
    model: &'a StructuralModel,
    selection: Option<Selection>,
    analysis: &'a AnalysisState,
) -> Element<'a, Message> {
    column![
        text("Inspector").size(18).color(theme::TEXT),
        selection_panel(model, selection),
        analysis_panel(analysis),
    ]
    .spacing(16)
    .padding(14)
    .width(Fill)
    .into()
}

fn selection_panel(model: &StructuralModel, selection: Option<Selection>) -> Element<'_, Message> {
    match selection {
        Some(Selection::Node(id)) => node_panel(model, id),
        Some(Selection::Element(id)) => member_panel(model, id),
        None => empty_panel("Selection", "Nothing selected"),
    }
}

fn node_panel(model: &StructuralModel, id: usize) -> Element<'_, Message> {
    let Some(node) = model.node(id) else {
        return empty_panel("Selection", "Missing node");
    };

    panel(
        "Node",
        column![
            property("Id", format!("N{}", node.id)),
            property("X", format!("{:.3} m", node.x)),
            property("Y", format!("{:.3} m", node.y)),
            property("Z", format!("{:.3} m", node.z)),
            property("Supports", support_summary(model, id)),
            property("Loads", load_summary(model, id)),
        ],
    )
}

fn member_panel(model: &StructuralModel, id: usize) -> Element<'_, Message> {
    let Some(element) = model.element(id) else {
        return empty_panel("Selection", "Missing member");
    };

    let (_, node_i, node_j) = element_data(element);
    let length = member_length(model, node_i, node_j).unwrap_or_default();

    panel(
        "Member",
        column![
            property("Id", format!("M{id}")),
            property("Type", element_kind(element).to_string()),
            property("Start", format!("N{node_i}")),
            property("End", format!("N{node_j}")),
            property("Length", format!("{length:.3} m")),
        ],
    )
}

fn analysis_panel(analysis: &AnalysisState) -> Element<'_, Message> {
    match analysis {
        AnalysisState::Idle => empty_panel("Analysis", "Not solved"),
        AnalysisState::Success(summary) => panel(
            "Analysis",
            column![
                property("Scope", summary.result_scope.to_string()),
                property("Max |u|", format!("{:.3e} m", summary.max_displacement)),
                property("Reactions", summary.reaction_count.to_string()),
            ],
        ),
        AnalysisState::Failed(error) => panel(
            "Analysis",
            column![text(error).size(14).color(theme::LOAD).width(Fill)],
        ),
    }
}

fn panel<'a>(
    title: &'static str,
    content: iced::widget::Column<'a, Message>,
) -> Element<'a, Message> {
    container(column![text(title).size(15).color(theme::TEXT), content.spacing(8)].spacing(10))
        .padding(10)
        .width(Fill)
        .style(theme::inset)
        .into()
}

fn empty_panel(title: &'static str, value: &'static str) -> Element<'static, Message> {
    panel(
        title,
        column![text(value).size(14).color(theme::TEXT_MUTED).width(Fill)],
    )
}

fn property(label: &'static str, value: String) -> Element<'static, Message> {
    row![
        text(label)
            .size(13)
            .color(theme::TEXT_MUTED)
            .width(Length::Fixed(92.0)),
        text(value).size(14).color(theme::TEXT).width(Fill),
    ]
    .spacing(8)
    .into()
}

fn support_summary(model: &StructuralModel, node_id: usize) -> String {
    let labels = model
        .supports
        .iter()
        .filter(|support| support.node_id == node_id)
        .map(|support| dof_label(support.dof))
        .collect::<Vec<_>>();

    if labels.is_empty() {
        "None".to_string()
    } else {
        labels.join(", ")
    }
}

fn load_summary(model: &StructuralModel, node_id: usize) -> String {
    let labels = model
        .nodal_loads
        .iter()
        .filter(|load| load.node_id == node_id)
        .map(|load| format!("{} {:+.1} kN", dof_label(load.dof), load.magnitude / 1000.0))
        .collect::<Vec<_>>();

    if labels.is_empty() {
        "None".to_string()
    } else {
        labels.join(", ")
    }
}
