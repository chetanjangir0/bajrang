use std::collections::BTreeMap;

use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Element, Fill, Length};
use model::dof::Dof;

use crate::{
    app::Message,
    state::{
        CoordinateAxis, InteractionDraft, LoadField, MemberEndpoint, Selection, StructuralModel,
        SupportBuilder, SupportPreset, WorkspaceTool, dof_label, element_data, element_id,
        element_kind,
    },
    theme,
};

pub fn view<'a>(
    model: &'a StructuralModel,
    selection: Option<Selection>,
    tool: WorkspaceTool,
    draft: InteractionDraft,
    node_coordinate_edits: &'a BTreeMap<(usize, CoordinateAxis), String>,
    member_endpoint_edits: &'a BTreeMap<(usize, MemberEndpoint), String>,
    load_edits: &'a BTreeMap<(usize, LoadField), String>,
    support_builder: Option<SupportBuilder>,
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
        tree = tree.push(members(
            model,
            selection,
            filter.edit_members(),
            member_endpoint_edits,
        ));
    }

    if filter.show_supports {
        tree = tree.push(supports(
            model,
            selection,
            filter.edit_supports(),
            support_builder,
        ));
    }

    if filter.show_loads {
        tree = tree.push(loads(model, filter.edit_loads(), load_edits));
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
            WorkspaceTool::Analyze => Self {
                tool,
                show_nodes: true,
                show_members: true,
                show_supports: true,
                show_loads: true,
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
            WorkspaceTool::Analyze => "Model",
        }
    }

    fn edit_nodes(self) -> bool {
        self.tool == WorkspaceTool::AddNode
    }

    fn edit_members(self) -> bool {
        self.tool == WorkspaceTool::DrawMember
    }

    fn edit_loads(self) -> bool {
        self.tool == WorkspaceTool::AssignLoad
    }

    fn edit_supports(self) -> bool {
        self.tool == WorkspaceTool::AssignSupport
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
        return empty_section(
            "Nodes",
            "No nodes in this model",
            add_message(editable, EntityKind::Node),
        );
    }

    model
        .nodes
        .iter()
        .fold(
            section("Nodes", add_message(editable, EntityKind::Node)),
            |column, node| {
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
            },
        )
        .into()
}

fn members<'a>(
    model: &'a StructuralModel,
    selection: Option<Selection>,
    editable: bool,
    member_endpoint_edits: &'a BTreeMap<(usize, MemberEndpoint), String>,
) -> Element<'a, Message> {
    if model.elements.is_empty() {
        return empty_section(
            "Members",
            "No members in this model",
            add_message(editable, EntityKind::Member),
        );
    }

    model
        .elements
        .iter()
        .fold(
            section("Members", add_message(editable, EntityKind::Member)),
            |column, element| {
                let (id, node_i, node_j) = element_data(element);

                if editable {
                    column.push(editable_member_row(
                        id,
                        node_i,
                        node_j,
                        selection == Some(Selection::Element(element_id(element))),
                        member_endpoint_edits,
                    ))
                } else {
                    column.push(selectable_row(
                        format!("M{id}"),
                        format!("{}  N{}-N{}", element_kind(element), node_i, node_j),
                        selection == Some(Selection::Element(element_id(element))),
                        Selection::Element(id),
                    ))
                }
            },
        )
        .into()
}

fn supports<'a>(
    model: &'a StructuralModel,
    selection: Option<Selection>,
    editable: bool,
    support_builder: Option<SupportBuilder>,
) -> Element<'a, Message> {
    let mut section = section("Supports", add_message(editable, EntityKind::Support));

    if editable {
        if let Some(builder) = support_builder {
            section = section.push(support_builder_panel(builder));
        } else {
            section = section.push(
                container(
                    text("Select a node, then use + to assign a support")
                        .size(13)
                        .color(theme::TEXT_MUTED),
                )
                .padding([7, 8])
                .width(Fill)
                .style(theme::neutral_row),
            );
        }
    }

    if model.supports.is_empty() {
        return section
            .push(
                container(
                    text("No supports assigned")
                        .size(13)
                        .color(theme::TEXT_MUTED),
                )
                .padding([7, 8])
                .width(Fill)
                .style(theme::neutral_row),
            )
            .into();
    }

    grouped_supports(model)
        .into_iter()
        .fold(section, |column, group| {
            column.push(support_group_row(group, selection, editable))
        })
        .into()
}

fn loads<'a>(
    model: &'a StructuralModel,
    editable: bool,
    load_edits: &'a BTreeMap<(usize, LoadField), String>,
) -> Element<'a, Message> {
    if model.nodal_loads.is_empty() {
        return empty_section(
            "Loads",
            "No nodal loads assigned",
            add_message(editable, EntityKind::Load),
        );
    }

    model
        .nodal_loads
        .iter()
        .enumerate()
        .fold(
            section("Loads", add_message(editable, EntityKind::Load)),
            |column, (index, load)| {
                if editable {
                    column.push(editable_load_row(
                        index,
                        load.node_id,
                        dof_label(load.dof),
                        load.magnitude / 1000.0,
                        load_edits,
                    ))
                } else {
                    column.push(static_row(
                        format!("N{}", load.node_id),
                        format!("{} {:+.1} kN", dof_label(load.dof), load.magnitude / 1000.0),
                    ))
                }
            },
        )
        .into()
}

fn panel_title(label: &str) -> Element<'_, Message> {
    text(label).size(18).color(theme::TEXT).into()
}

fn section(label: &str, add: Option<Message>) -> iced::widget::Column<'_, Message> {
    column![section_header(label, add)].spacing(4).width(Fill)
}

fn empty_section(
    title: &'static str,
    message: &'static str,
    add: Option<Message>,
) -> Element<'static, Message> {
    section(title, add)
        .push(
            container(text(message).size(13).color(theme::TEXT_MUTED).width(Fill))
                .padding([7, 8])
                .width(Fill)
                .style(theme::neutral_row),
        )
        .into()
}

fn section_header(label: &str, add: Option<Message>) -> Element<'_, Message> {
    let mut header = row![text(label).size(13).color(theme::TEXT_MUTED).width(Fill),]
        .spacing(6)
        .align_y(Alignment::Center);

    if let Some(message) = add {
        header = header.push(
            button(text("+").size(14).color(theme::TEXT))
                .padding([2, 8])
                .style(theme::secondary_button)
                .on_press(message),
        );
    }

    header.into()
}

#[derive(Debug, Clone, Copy)]
enum EntityKind {
    Node,
    Member,
    Load,
    Support,
}

fn add_message(editable: bool, kind: EntityKind) -> Option<Message> {
    editable.then(|| match kind {
        EntityKind::Node => Message::AddNodeRequested,
        EntityKind::Member => Message::AddMemberRequested,
        EntityKind::Load => Message::AddLoadRequested,
        EntityKind::Support => Message::AddSupportRequested,
    })
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
    let content = row![
        button(
            text(node_id.to_string())
                .size(14)
                .color(theme::TEXT)
                .width(Fill),
        )
        .style(if selected {
            theme::tool_button_active
        } else {
            theme::tool_button
        })
        .padding([4, 6])
        .width(Length::Fixed(28.0))
        .on_press(Message::SelectionRequested(Some(Selection::Node(node_id)))),
        coordinate_input(node_id, CoordinateAxis::X, x, node_coordinate_edits),
        coordinate_input(node_id, CoordinateAxis::Y, y, node_coordinate_edits),
        coordinate_input(node_id, CoordinateAxis::Z, z, node_coordinate_edits),
        delete_button(Message::DeleteNodeRequested(node_id)),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    container(content)
        .padding([4, 6])
        .width(Fill)
        .style(theme::neutral_row)
        .into()
}

fn editable_member_row(
    element_id: usize,
    node_i: usize,
    node_j: usize,
    selected: bool,
    member_endpoint_edits: &BTreeMap<(usize, MemberEndpoint), String>,
) -> Element<'static, Message> {
    let content = row![
        button(
            text(element_id.to_string())
                .size(14)
                .color(theme::TEXT)
                .width(Fill),
        )
        .style(if selected {
            theme::tool_button_active
        } else {
            theme::tool_button
        })
        .padding([4, 6])
        .width(Length::Fixed(28.0))
        .on_press(Message::SelectionRequested(Some(Selection::Element(
            element_id
        )))),
        member_endpoint_input(
            element_id,
            MemberEndpoint::Start,
            node_i,
            member_endpoint_edits
        ),
        member_endpoint_input(
            element_id,
            MemberEndpoint::End,
            node_j,
            member_endpoint_edits
        ),
        delete_button(Message::DeleteMemberRequested(element_id)),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    container(content)
        .padding([4, 6])
        .width(Fill)
        .style(theme::neutral_row)
        .into()
}

fn editable_load_row(
    index: usize,
    node_id: usize,
    dof: &'static str,
    magnitude_kn: f64,
    load_edits: &BTreeMap<(usize, LoadField), String>,
) -> Element<'static, Message> {
    let content = row![
        text(index.to_string())
            .size(14)
            .color(theme::TEXT)
            .width(Length::Fixed(28.0)),
        load_input(index, LoadField::Node, node_id.to_string(), load_edits),
        load_input(index, LoadField::Dof, dof.to_string(), load_edits),
        load_input(
            index,
            LoadField::Magnitude,
            coordinate_value(magnitude_kn),
            load_edits
        ),
        delete_button(Message::DeleteLoadRequested(index)),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    container(content)
        .padding([4, 6])
        .width(Fill)
        .style(theme::neutral_row)
        .into()
}

#[derive(Debug, Clone)]
struct SupportGroup {
    node_id: usize,
    dofs: Vec<Dof>,
}

fn grouped_supports(model: &StructuralModel) -> Vec<SupportGroup> {
    let mut groups: BTreeMap<usize, Vec<Dof>> = BTreeMap::new();

    for support in &model.supports {
        groups.entry(support.node_id).or_default().push(support.dof);
    }

    groups
        .into_iter()
        .map(|(node_id, mut dofs)| {
            dofs.sort_by_key(|dof| dof_order(*dof));
            SupportGroup { node_id, dofs }
        })
        .collect()
}

fn support_group_row(
    group: SupportGroup,
    selection: Option<Selection>,
    editable: bool,
) -> Element<'static, Message> {
    let selected = selection == Some(Selection::Node(group.node_id));
    let node_id = group.node_id;
    let detail = support_group_detail(&group);

    let content = row![
        button(
            text(format!("N{node_id}"))
                .size(14)
                .color(theme::TEXT)
                .width(Fill),
        )
        .style(if selected {
            theme::tool_button_active
        } else {
            theme::tool_button
        })
        .padding([4, 6])
        .width(Length::Fixed(44.0))
        .on_press(Message::SelectionRequested(Some(Selection::Node(node_id)))),
        column![
            text(support_group_kind(&group.dofs))
                .size(14)
                .color(theme::TEXT),
            text(detail).size(12).color(theme::TEXT_MUTED),
        ]
        .spacing(1)
        .width(Fill),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let content = if editable {
        content.push(delete_button(Message::DeleteSupportGroupRequested(node_id)))
    } else {
        content
    };

    container(content)
        .padding([4, 6])
        .width(Fill)
        .style(theme::neutral_row)
        .into()
}

fn support_builder_panel(builder: SupportBuilder) -> Element<'static, Message> {
    let presets = row![
        support_preset_button(SupportPreset::Pin),
        support_preset_button(SupportPreset::Fixed),
        support_preset_button(SupportPreset::Roller),
    ]
    .spacing(6)
    .width(Fill);

    let translations = row![
        dof_toggle("Ux", Dof::Ux, builder.ux),
        dof_toggle("Uy", Dof::Uy, builder.uy),
        dof_toggle("Uz", Dof::Uz, builder.uz),
    ]
    .spacing(6)
    .width(Fill);

    let rotations = row![
        dof_toggle("Rx", Dof::Rx, builder.rx),
        dof_toggle("Ry", Dof::Ry, builder.ry),
        dof_toggle("Rz", Dof::Rz, builder.rz),
    ]
    .spacing(6)
    .width(Fill);

    container(
        column![
            row![
                text(format!("N{} support", builder.node_id))
                    .size(14)
                    .color(theme::TEXT)
                    .width(Fill),
                button(text("Cancel").size(12).color(theme::TEXT))
                    .padding([3, 8])
                    .style(theme::secondary_button)
                    .on_press(Message::CancelSupportRequested),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            text("Presets").size(12).color(theme::TEXT_MUTED),
            presets,
            text("Custom restraints").size(12).color(theme::TEXT_MUTED),
            translations,
            rotations,
            button(text("Apply custom").size(13).color(theme::TEXT))
                .padding([6, 10])
                .width(Fill)
                .style(theme::primary_button)
                .on_press(Message::ApplyCustomSupportRequested),
        ]
        .spacing(7),
    )
    .padding(9)
    .width(Fill)
    .style(theme::inset)
    .into()
}

fn support_preset_button(preset: SupportPreset) -> Element<'static, Message> {
    button(text(preset.label()).size(13).color(theme::TEXT))
        .padding([6, 8])
        .width(Length::Fill)
        .style(theme::secondary_button)
        .on_press(Message::AddSupportPresetRequested(preset))
        .into()
}

fn dof_toggle(label: &'static str, dof: Dof, active: bool) -> Element<'static, Message> {
    button(text(label).size(13).color(theme::TEXT))
        .padding([6, 8])
        .width(Length::Fill)
        .style(if active {
            theme::tool_button_active
        } else {
            theme::secondary_button
        })
        .on_press(Message::CustomSupportDofToggled {
            dof,
            restrained: !active,
        })
        .into()
}

fn support_group_kind(dofs: &[Dof]) -> &'static str {
    if dofs_match(dofs, &[Dof::Ux, Dof::Uy, Dof::Uz]) {
        "Pin"
    } else if dofs_match(
        dofs,
        &[Dof::Ux, Dof::Uy, Dof::Uz, Dof::Rx, Dof::Ry, Dof::Rz],
    ) {
        "Fixed"
    } else if dofs_match(dofs, &[Dof::Uy]) {
        "Roller"
    } else {
        "Custom"
    }
}

fn support_group_detail(group: &SupportGroup) -> String {
    group
        .dofs
        .iter()
        .map(|dof| dof_label(*dof))
        .collect::<Vec<_>>()
        .join(", ")
}

fn dofs_match(actual: &[Dof], expected: &[Dof]) -> bool {
    actual.len() == expected.len() && expected.iter().all(|dof| actual.contains(dof))
}

fn dof_order(dof: Dof) -> usize {
    match dof {
        Dof::Ux => 0,
        Dof::Uy => 1,
        Dof::Uz => 2,
        Dof::Rx => 3,
        Dof::Ry => 4,
        Dof::Rz => 5,
    }
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

fn member_endpoint_input(
    element_id: usize,
    endpoint: MemberEndpoint,
    node_id: usize,
    member_endpoint_edits: &BTreeMap<(usize, MemberEndpoint), String>,
) -> Element<'static, Message> {
    let value = member_endpoint_edits
        .get(&(element_id, endpoint))
        .cloned()
        .unwrap_or_else(|| node_id.to_string());

    text_input(endpoint.label(), &value)
        .on_input(move |value| Message::MemberEndpointDraftChanged {
            element_id,
            endpoint,
            value,
        })
        .on_submit(Message::MemberEndpointSubmitted {
            element_id,
            endpoint,
        })
        .padding([4, 6])
        .size(13)
        .width(Length::Fill)
        .style(theme::compact_input)
        .into()
}

fn load_input(
    index: usize,
    field: LoadField,
    fallback: String,
    load_edits: &BTreeMap<(usize, LoadField), String>,
) -> Element<'static, Message> {
    let value = load_edits.get(&(index, field)).cloned().unwrap_or(fallback);

    text_input(field.label(), &value)
        .on_input(move |value| Message::LoadDraftChanged {
            index,
            field,
            value,
        })
        .on_submit(Message::LoadSubmitted { index })
        .padding([4, 6])
        .size(13)
        .width(Length::Fill)
        .style(theme::compact_input)
        .into()
}

fn delete_button(message: Message) -> Element<'static, Message> {
    button(text("x").size(13).color(theme::TEXT))
        .padding([4, 7])
        .width(Length::Fixed(28.0))
        .style(theme::secondary_button)
        .on_press(message)
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
