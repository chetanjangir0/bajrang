use std::collections::BTreeMap;

use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Element, Fill, Length};

use crate::{
    app::Message,
    state::{
        CoordinateAxis, InteractionDraft, Selection, StructuralModel, WorkspaceTool, dof_label,
        element_data, element_id, element_kind,
    },
    theme,
};

pub fn view<'a>(
    model: &'a StructuralModel,
    selection: Option<Selection>,
    tool: WorkspaceTool,
    draft: InteractionDraft,
    node_coordinate_edits: &'a BTreeMap<(usize, CoordinateAxis), String>,
) -> Element<'a, Message> {
    let filter = ModelTreeFilter::for_tool(tool);
    let mut tree = column![panel_title(filter.title()), summary(model, filter),]
        .spacing(16)
        .padding(14)
        .width(Fill);

    if filter.show_nodes {
        tree = tree.push(nodes(
            model,
            selection,
            draft,
            filter.edit_nodes(),
            node_coordinate_edits,
        ));
    }

    if filter.show_members {
        tree = tree.push(members(model, selection));
    }

    if filter.show_supports {
        tree = tree.push(supports(model));
    }

    if filter.show_loads {
        tree = tree.push(loads(model));
    }

    tree.into()
}

#[derive(Debug, Clone, Copy)]
struct ModelTreeFilter {
    tool: WorkspaceTool,
    show_nodes: bool,
    show_members: bool,
    show_supports: bool,
    show_loads: bool,
}

impl ModelTreeFilter {
    fn for_tool(tool: WorkspaceTool) -> Self {
        match tool {
            WorkspaceTool::Select => Self {
                tool,
                show_nodes: true,
                show_members: true,
                show_supports: true,
                show_loads: true,
            },
            WorkspaceTool::AddNode => Self {
                tool,
                show_nodes: true,
                show_members: false,
                show_supports: false,
                show_loads: false,
            },
            WorkspaceTool::DrawMember => Self {
                tool,
                show_nodes: false,
                show_members: true,
                show_supports: false,
                show_loads: false,
            },
            WorkspaceTool::AssignLoad => Self {
                tool,
                show_nodes: false,
                show_members: false,
                show_supports: false,
                show_loads: true,
            },
            WorkspaceTool::AssignSupport => Self {
                tool,
                show_nodes: false,
                show_members: false,
                show_supports: true,
                show_loads: false,
            },
        }
    }

    fn title(self) -> &'static str {
        match self.tool {
            WorkspaceTool::Select => "Model",
            WorkspaceTool::AddNode => "Nodes",
            WorkspaceTool::DrawMember => "Members",
            WorkspaceTool::AssignLoad => "Loads",
            WorkspaceTool::AssignSupport => "Supports",
        }
    }

    fn edit_nodes(self) -> bool {
        self.tool == WorkspaceTool::AddNode
    }
}

fn summary(model: &StructuralModel, filter: ModelTreeFilter) -> Element<'_, Message> {
    let mut metrics = column![].spacing(6);

    if filter.show_nodes {
        metrics = metrics.push(metric_row("Nodes", model.nodes.len()));
    }

    if filter.show_members {
        metrics = metrics.push(metric_row("Members", model.elements.len()));
    }

    if filter.show_supports {
        metrics = metrics.push(metric_row("Supports", model.supports.len()));
    }

    if filter.show_loads {
        metrics = metrics.push(metric_row("Loads", model.nodal_loads.len()));
    }

    container(metrics)
        .padding(10)
        .width(Fill)
        .style(theme::inset)
        .into()
}

fn nodes<'a>(
    model: &'a StructuralModel,
    selection: Option<Selection>,
    draft: InteractionDraft,
    editable: bool,
    node_coordinate_edits: &'a BTreeMap<(usize, CoordinateAxis), String>,
) -> Element<'a, Message> {
    if model.nodes.is_empty() {
        return empty_section("Nodes", "No nodes in this model");
    }

    model
        .nodes
        .iter()
        .fold(section("Nodes"), |column, node| {
            let selected = selection == Some(Selection::Node(node.id));
            let active_draft = draft.member_start == Some(node.id);

            if editable {
                column.push(editable_node_row(
                    node.id,
                    node.x,
                    node.y,
                    node.z,
                    selected || active_draft,
                    node_coordinate_edits,
                ))
            } else {
                column.push(selectable_row(
                    format!("N{}", node.id),
                    format!("{:.2}, {:.2}", node.x, node.y),
                    selected || active_draft,
                    Selection::Node(node.id),
                ))
            }
        })
        .into()
}

fn members(model: &StructuralModel, selection: Option<Selection>) -> Element<'_, Message> {
    if model.elements.is_empty() {
        return empty_section("Members", "No members in this model");
    }

    model
        .elements
        .iter()
        .fold(section("Members"), |column, element| {
            let (id, node_i, node_j) = element_data(element);

            column.push(selectable_row(
                format!("M{id}"),
                format!("{}  N{}-N{}", element_kind(element), node_i, node_j),
                selection == Some(Selection::Element(element_id(element))),
                Selection::Element(id),
            ))
        })
        .into()
}

fn supports(model: &StructuralModel) -> Element<'_, Message> {
    if model.supports.is_empty() {
        return empty_section("Supports", "No supports assigned");
    }

    model
        .supports
        .iter()
        .fold(section("Supports"), |column, support| {
            column.push(static_row(
                format!("N{}", support.node_id),
                dof_label(support.dof).to_string(),
            ))
        })
        .into()
}

fn loads(model: &StructuralModel) -> Element<'_, Message> {
    if model.nodal_loads.is_empty() {
        return empty_section("Loads", "No nodal loads assigned");
    }

    model
        .nodal_loads
        .iter()
        .fold(section("Loads"), |column, load| {
            column.push(static_row(
                format!("N{}", load.node_id),
                format!("{} {:+.1} kN", dof_label(load.dof), load.magnitude / 1000.0),
            ))
        })
        .into()
}

fn panel_title(label: &str) -> Element<'_, Message> {
    text(label).size(18).color(theme::TEXT).into()
}

fn section(label: &str) -> iced::widget::Column<'_, Message> {
    column![text(label).size(13).color(theme::TEXT_MUTED)]
        .spacing(4)
        .width(Fill)
}

fn empty_section(title: &'static str, message: &'static str) -> Element<'static, Message> {
    section(title)
        .push(
            container(text(message).size(13).color(theme::TEXT_MUTED).width(Fill))
                .padding([7, 8])
                .width(Fill)
                .style(theme::neutral_row),
        )
        .into()
}

fn metric_row(label: &str, value: usize) -> Element<'_, Message> {
    row![
        text(label).size(13).color(theme::TEXT_MUTED).width(Fill),
        text(value).size(14).color(theme::TEXT),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn selectable_row(
    label: String,
    detail: String,
    selected: bool,
    selection: Selection,
) -> Element<'static, Message> {
    let content = row![
        text(label)
            .size(14)
            .color(theme::TEXT)
            .width(Length::Fixed(44.0)),
        text(detail).size(13).color(theme::TEXT_MUTED).width(Fill),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    button(container(content).padding([6, 8]).width(Fill))
        .style(if selected {
            theme::tool_button_active
        } else {
            theme::tool_button
        })
        .padding(0)
        .width(Fill)
        .on_press(Message::SelectionRequested(Some(selection)))
        .into()
}

fn editable_node_row(
    node_id: usize,
    x: f64,
    y: f64,
    z: f64,
    selected: bool,
    node_coordinate_edits: &BTreeMap<(usize, CoordinateAxis), String>,
) -> Element<'static, Message> {
    let content = column![
        button(
            row![
                text(format!("N{node_id}"))
                    .size(14)
                    .color(theme::TEXT)
                    .width(Length::Fixed(44.0)),
                text("Coordinates")
                    .size(13)
                    .color(theme::TEXT_MUTED)
                    .width(Fill),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .style(if selected {
            theme::tool_button_active
        } else {
            theme::tool_button
        })
        .padding([5, 8])
        .width(Fill)
        .on_press(Message::SelectionRequested(Some(Selection::Node(node_id)))),
        row![
            coordinate_input(node_id, CoordinateAxis::X, x, node_coordinate_edits),
            coordinate_input(node_id, CoordinateAxis::Y, y, node_coordinate_edits),
            coordinate_input(node_id, CoordinateAxis::Z, z, node_coordinate_edits),
        ]
        .spacing(6)
        .width(Fill),
    ]
    .spacing(6);

    container(content)
        .padding([6, 8])
        .width(Fill)
        .style(theme::neutral_row)
        .into()
}

fn coordinate_input(
    node_id: usize,
    axis: CoordinateAxis,
    value: f64,
    node_coordinate_edits: &BTreeMap<(usize, CoordinateAxis), String>,
) -> Element<'static, Message> {
    let value = node_coordinate_edits
        .get(&(node_id, axis))
        .cloned()
        .unwrap_or_else(|| coordinate_value(value));

    text_input(axis.label(), &value)
        .on_input(move |value| Message::NodeCoordinateDraftChanged {
            node_id,
            axis,
            value,
        })
        .on_submit(Message::NodeCoordinateSubmitted { node_id, axis })
        .padding([4, 6])
        .size(13)
        .width(Length::Fill)
        .style(theme::compact_input)
        .into()
}

fn coordinate_value(value: f64) -> String {
    if value == 0.0 {
        "0".to_string()
    } else {
        format!("{value:.3}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

fn static_row(label: String, detail: String) -> Element<'static, Message> {
    container(
        row![
            text(label)
                .size(14)
                .color(theme::TEXT)
                .width(Length::Fixed(44.0)),
            text(detail).size(13).color(theme::TEXT_MUTED).width(Fill),
        ]
        .spacing(8),
    )
    .padding([4, 8])
    .width(Fill)
    .style(theme::neutral_row)
    .into()
}
