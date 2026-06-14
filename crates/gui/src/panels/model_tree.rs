use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Fill, Length};

use crate::{
    app::Message,
    state::{
        InteractionDraft, Selection, StructuralModel, dof_label, element_data, element_id,
        element_kind,
    },
    theme,
};

pub fn view(
    model: &StructuralModel,
    selection: Option<Selection>,
    draft: InteractionDraft,
) -> Element<'_, Message> {
    column![
        panel_title("Model"),
        summary(model),
        nodes(model, selection, draft),
        members(model, selection),
        supports(model),
        loads(model),
    ]
    .spacing(16)
    .padding(14)
    .width(Fill)
    .into()
}

fn summary(model: &StructuralModel) -> Element<'_, Message> {
    container(
        column![
            metric_row("Nodes", model.nodes.len()),
            metric_row("Members", model.elements.len()),
            metric_row("Supports", model.supports.len()),
            metric_row("Loads", model.nodal_loads.len()),
        ]
        .spacing(6),
    )
    .padding(10)
    .width(Fill)
    .style(theme::inset)
    .into()
}

fn nodes(
    model: &StructuralModel,
    selection: Option<Selection>,
    draft: InteractionDraft,
) -> Element<'_, Message> {
    model
        .nodes
        .iter()
        .fold(section("Nodes"), |column, node| {
            let selected = selection == Some(Selection::Node(node.id));
            let active_draft = draft.member_start == Some(node.id);

            column.push(selectable_row(
                format!("N{}", node.id),
                format!("{:.2}, {:.2}", node.x, node.y),
                selected || active_draft,
                Selection::Node(node.id),
            ))
        })
        .into()
}

fn members(model: &StructuralModel, selection: Option<Selection>) -> Element<'_, Message> {
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
