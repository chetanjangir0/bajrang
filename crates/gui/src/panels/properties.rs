use iced::widget::{column, container, row, text};
use iced::{Element, Fill, Length};
use model::dof::Dof;

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
        analysis_panel(model, analysis, selection),
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

fn analysis_panel<'a>(
    model: &'a StructuralModel,
    analysis: &'a AnalysisState,
    selection: Option<Selection>,
) -> Element<'a, Message> {
    match analysis {
        AnalysisState::Idle => empty_panel("Analysis", "Not solved"),
        AnalysisState::Success(summary) => panel("Analysis", {
            let mut content = column![
                property("Scope", summary.result_scope.to_string()),
                property("Max |u|", format!("{:.3e} m", summary.max_displacement)),
                property("Reactions", summary.reaction_count.to_string()),
                property(
                    "Max |R|",
                    format!("{:.3} kN", summary.max_reaction / 1000.0)
                ),
                property(
                    "Max member",
                    format!("{:.3} kN", summary.max_member_force / 1000.0)
                ),
            ];

            if let Some(Selection::Node(node_id)) = selection {
                content = content.push(node_results(summary, node_id));
            }

            if let Some(Selection::Element(element_id)) = selection {
                if model.element(element_id).is_some() {
                    content = content.push(member_results(summary, element_id));
                }
            }

            content
        }),
        AnalysisState::Failed(error) => panel(
            "Analysis",
            column![text(error).size(14).color(theme::LOAD).width(Fill)],
        ),
    }
}

fn node_results(summary: &crate::state::AnalysisSummary, node_id: usize) -> Element<'_, Message> {
    column![
        property(
            "Ux",
            format!("{:.3e} m", displacement(summary, node_id, Dof::Ux))
        ),
        property(
            "Uy",
            format!("{:.3e} m", displacement(summary, node_id, Dof::Uy))
        ),
        property(
            "R Ux",
            format!("{:.3} kN", reaction(summary, node_id, Dof::Ux) / 1000.0)
        ),
        property(
            "R Uy",
            format!("{:.3} kN", reaction(summary, node_id, Dof::Uy) / 1000.0)
        ),
    ]
    .spacing(8)
    .into()
}

fn member_results(
    summary: &crate::state::AnalysisSummary,
    element_id: usize,
) -> Element<'_, Message> {
    let Some(result) = summary
        .member_results
        .iter()
        .find(|result| result.element_id == element_id)
    else {
        return text("No member result")
            .size(14)
            .color(theme::TEXT_MUTED)
            .into();
    };

    result
        .values
        .iter()
        .fold(
            column![property("Result", result.kind.to_string())].spacing(8),
            |column, (label, value)| {
                column.push(property(label, format!("{:.3} kN", value / 1000.0)))
            },
        )
        .into()
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

fn displacement(summary: &crate::state::AnalysisSummary, node_id: usize, dof: Dof) -> f64 {
    summary
        .displacements
        .get(model::dof::global_dof_index(node_id, dof))
        .copied()
        .unwrap_or_default()
}

fn reaction(summary: &crate::state::AnalysisSummary, node_id: usize, dof: Dof) -> f64 {
    summary
        .reactions
        .iter()
        .find(|reaction| reaction.node_id == node_id && reaction.dof == dof)
        .map_or(0.0, |reaction| reaction.magnitude)
}
