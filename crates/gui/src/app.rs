use std::collections::BTreeMap;

use iced::keyboard::{self, Key, key};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Fill, Length, Subscription, Task};
use model::{dof::Dof, load::DistributedLoadDirection};

use crate::{
    expression, panels,
    state::{
        AnalysisState, CoordinateAxis, InteractionDraft, LoadBuilder, LoadTarget, MemberEndpoint,
        ResultDisplay, Selection, StructuralModel, SupportBuilder, SupportPreset, WorkspaceTool,
        run_basic_analysis,
    },
    theme,
    viewport::{ViewportPress, ViewportState, ViewportUpdate},
};

#[derive(Debug)]
pub struct BajrangApp {
    pub model: StructuralModel,
    pub tool: WorkspaceTool,
    pub selection: Option<Selection>,
    pub draft: InteractionDraft,
    pub viewport: ViewportState,
    pub analysis: AnalysisState,
    pub result_display: ResultDisplay,
    pub result_scale: f64,
    pub status: StatusLine,
    pub node_coordinate_edits: BTreeMap<(usize, CoordinateAxis), String>,
    pub member_endpoint_edits: BTreeMap<(usize, MemberEndpoint), String>,
    pub load_builder: Option<LoadBuilder>,
    pub support_builder: Option<SupportBuilder>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToolSelected(WorkspaceTool),
    SelectionRequested(Option<Selection>),
    NodeCoordinateDraftChanged {
        node_id: usize,
        axis: CoordinateAxis,
        value: String,
    },
    NodeCoordinateSubmitted {
        node_id: usize,
        axis: CoordinateAxis,
    },
    MemberEndpointDraftChanged {
        element_id: usize,
        endpoint: MemberEndpoint,
        value: String,
    },
    MemberEndpointSubmitted {
        element_id: usize,
        endpoint: MemberEndpoint,
    },
    AddNodeRequested,
    AddMemberRequested,
    AddLoadRequested,
    LoadPointDofSelected(Dof),
    LoadDistributedDirectionSelected(DistributedLoadDirection),
    LoadMagnitudeChanged(String),
    ApplyLoadRequested,
    CancelLoadRequested,
    AddSupportRequested,
    AddSupportPresetRequested(SupportPreset),
    CustomSupportDofToggled {
        dof: Dof,
        restrained: bool,
    },
    ApplyCustomSupportRequested,
    CancelSupportRequested,
    DeleteNodeRequested(usize),
    DeleteMemberRequested(usize),
    DeletePointLoadRequested(usize),
    DeleteDistributedLoadRequested(usize),
    DeleteSupportGroupRequested(usize),
    ViewportPressed(ViewportPress),
    ViewportChanged(ViewportUpdate),
    SolveRequested,
    ResultDisplaySelected(ResultDisplay),
    ResultScaleChanged(f64),
    FocusNextInput,
    FocusPreviousInput,
    FitView,
    NewModel,
    LoadSample,
}

#[derive(Debug, Clone)]
pub struct StatusLine {
    pub text: String,
    pub level: StatusLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLevel {
    Neutral,
    Success,
    Warning,
    Error,
}

impl Default for BajrangApp {
    fn default() -> Self {
        Self {
            model: StructuralModel::sample(),
            tool: WorkspaceTool::Select,
            selection: None,
            draft: InteractionDraft::default(),
            viewport: ViewportState::default(),
            analysis: AnalysisState::Idle,
            result_display: ResultDisplay::Model,
            result_scale: 80.0,
            status: StatusLine::neutral("Ready"),
            node_coordinate_edits: BTreeMap::new(),
            member_endpoint_edits: BTreeMap::new(),
            load_builder: None,
            support_builder: None,
        }
    }
}

impl BajrangApp {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToolSelected(tool) => {
                self.tool = tool;
                self.draft.clear();
                self.clear_edit_drafts();
                self.set_status(
                    StatusLevel::Neutral,
                    format!("{} tool active", tool.label()),
                );
            }
            Message::SelectionRequested(selection) => {
                self.selection = selection;
                self.draft.clear();
                self.set_status(
                    StatusLevel::Neutral,
                    self.selection
                        .map_or_else(|| "Selection cleared".to_string(), Selection::label),
                );
            }
            Message::NodeCoordinateDraftChanged {
                node_id,
                axis,
                value,
            } => {
                self.node_coordinate_edits.insert((node_id, axis), value);
                self.selection = Some(Selection::Node(node_id));
            }
            Message::NodeCoordinateSubmitted { node_id, axis } => {
                self.handle_node_coordinate_submit(node_id, axis);
            }
            Message::MemberEndpointDraftChanged {
                element_id,
                endpoint,
                value,
            } => {
                self.member_endpoint_edits
                    .insert((element_id, endpoint), value);
                self.selection = Some(Selection::Element(element_id));
            }
            Message::MemberEndpointSubmitted {
                element_id,
                endpoint,
            } => self.handle_member_endpoint_submit(element_id, endpoint),
            Message::AddNodeRequested => self.add_node_from_tree(),
            Message::AddMemberRequested => self.add_member_from_tree(),
            Message::AddLoadRequested => self.add_load_from_tree(),
            Message::LoadPointDofSelected(dof) => {
                if let Some(builder) = &mut self.load_builder {
                    builder.dof = dof;
                }
            }
            Message::LoadDistributedDirectionSelected(direction) => {
                if let Some(builder) = &mut self.load_builder {
                    builder.direction = direction;
                }
            }
            Message::LoadMagnitudeChanged(value) => {
                if let Some(builder) = &mut self.load_builder {
                    builder.magnitude = value;
                }
            }
            Message::ApplyLoadRequested => self.apply_load(),
            Message::CancelLoadRequested => {
                self.load_builder = None;
                self.set_status(StatusLevel::Neutral, "Load assignment cancelled");
            }
            Message::AddSupportRequested => self.add_support_from_tree(),
            Message::AddSupportPresetRequested(preset) => self.add_support_preset(preset),
            Message::CustomSupportDofToggled { dof, restrained } => {
                self.toggle_custom_support_dof(dof, restrained);
            }
            Message::ApplyCustomSupportRequested => self.apply_custom_support(),
            Message::CancelSupportRequested => {
                self.support_builder = None;
                self.set_status(StatusLevel::Neutral, "Support assignment cancelled");
            }
            Message::DeleteNodeRequested(node_id) => self.delete_node(node_id),
            Message::DeleteMemberRequested(element_id) => self.delete_member(element_id),
            Message::DeletePointLoadRequested(index) => self.delete_point_load(index),
            Message::DeleteDistributedLoadRequested(index) => self.delete_distributed_load(index),
            Message::DeleteSupportGroupRequested(node_id) => self.delete_support_group(node_id),
            Message::ViewportPressed(press) => self.handle_viewport_press(press),
            Message::ViewportChanged(update) => self.viewport.apply(update),
            Message::SolveRequested => self.solve(),
            Message::ResultDisplaySelected(display) => {
                self.result_display = display;
                self.set_status(StatusLevel::Neutral, format!("{} view", display.label()));
            }
            Message::ResultScaleChanged(scale) => {
                self.result_scale = scale.clamp(0.0, 400.0);
                self.set_status(
                    StatusLevel::Neutral,
                    format!("Result scale {:.0}", self.result_scale),
                );
            }
            Message::FocusNextInput => {
                return iced::widget::operation::focus_next();
            }
            Message::FocusPreviousInput => {
                return iced::widget::operation::focus_previous();
            }
            Message::FitView => {
                self.viewport = ViewportState::default();
                self.set_status(StatusLevel::Neutral, "View reset");
            }
            Message::NewModel => {
                self.model = StructuralModel::empty();
                self.selection = None;
                self.draft.clear();
                self.clear_edit_drafts();
                self.analysis = AnalysisState::Idle;
                self.result_display = ResultDisplay::Model;
                self.set_status(StatusLevel::Neutral, "New model");
            }
            Message::LoadSample => {
                *self = Self::default();
            }
        }

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let sidebar_content = match self.tool {
            WorkspaceTool::Analyze => panels::analysis::view(
                &self.model,
                &self.analysis,
                self.result_display,
                self.result_scale,
            ),
            _ => panels::model_tree::view(
                &self.model,
                self.selection,
                self.tool,
                self.draft,
                &self.node_coordinate_edits,
                &self.member_endpoint_edits,
                self.load_builder.clone(),
                self.support_builder,
            ),
        };

        let sidebar = container(scrollable(sidebar_content))
            .width(292)
            .height(Fill)
            .style(theme::panel);

        let workspace = panels::workspace::view(
            &self.model,
            self.selection,
            self.tool,
            self.draft,
            self.viewport,
            &self.analysis,
            self.result_display,
            self.result_scale,
        );

        let inspector = container(scrollable(panels::properties::view(
            &self.model,
            self.selection,
            &self.analysis,
        )))
        .width(332)
        .height(Fill)
        .style(theme::panel);

        let body = row![sidebar, workspace, inspector].spacing(8).height(Fill);

        container(column![self.toolbar(), body, self.status_bar()].spacing(8))
            .padding(8)
            .width(Fill)
            .height(Fill)
            .style(theme::app_background)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed { key, modifiers, .. }
                if key == Key::Named(key::Named::Tab) =>
            {
                if modifiers.shift() {
                    Some(Message::FocusPreviousInput)
                } else {
                    Some(Message::FocusNextInput)
                }
            }
            _ => None,
        })
    }

    fn toolbar(&self) -> Element<'_, Message> {
        let tools = WorkspaceTool::ALL.into_iter().fold(
            row![].spacing(4).align_y(Alignment::Center),
            |row, tool| {
                row.push(
                    button(
                        column![text(tool.marker()).size(16), text(tool.label()).size(11)]
                            .align_x(Alignment::Center)
                            .spacing(1),
                    )
                    .width(72)
                    .height(54)
                    .padding(6)
                    .style(if tool == self.tool {
                        theme::tool_button_active
                    } else {
                        theme::tool_button
                    })
                    .on_press(Message::ToolSelected(tool)),
                )
            },
        );

        container(
            row![
                column![
                    text("Bajrang").size(22).color(theme::TEXT),
                    text(&self.model.name).size(13).color(theme::TEXT_MUTED),
                ]
                .spacing(2)
                .width(Length::FillPortion(2)),
                tools,
                row![
                    button(text("New").size(14))
                        .padding([8, 14])
                        .style(theme::secondary_button)
                        .on_press(Message::NewModel),
                    button(text("Sample").size(14))
                        .padding([8, 14])
                        .style(theme::secondary_button)
                        .on_press(Message::LoadSample),
                    button(text("Fit").size(14))
                        .padding([8, 14])
                        .style(theme::secondary_button)
                        .on_press(Message::FitView),
                ]
                .spacing(6)
                .align_y(Alignment::Center)
                .width(Length::Shrink),
            ]
            .spacing(20)
            .align_y(Alignment::Center),
        )
        .padding([10, 12])
        .width(Fill)
        .style(theme::panel)
        .into()
    }

    fn status_bar(&self) -> Element<'_, Message> {
        let selection = self
            .selection
            .map_or_else(|| "No selection".to_string(), Selection::label);

        container(
            row![
                text(&self.status.text)
                    .size(14)
                    .color(theme::status_color(self.status.level))
                    .width(Length::Fill),
                text(selection).size(14).color(theme::TEXT_MUTED),
                text(format!(
                    "{} nodes  {} members  {} supports  {} loads",
                    self.model.nodes.len(),
                    self.model.elements.len(),
                    self.model.supports.len(),
                    self.model.nodal_loads.len() + self.model.distributed_loads.len()
                ))
                .size(14)
                .color(theme::TEXT_MUTED),
            ]
            .spacing(18)
            .align_y(Alignment::Center),
        )
        .padding([8, 12])
        .width(Fill)
        .style(theme::status_bar)
        .into()
    }

    fn handle_viewport_press(&mut self, press: ViewportPress) {
        match self.tool {
            WorkspaceTool::Select | WorkspaceTool::Analyze => {
                self.selection = press.target;
                self.draft.clear();
                self.set_status(
                    StatusLevel::Neutral,
                    self.selection
                        .map_or_else(|| "Selection cleared".to_string(), Selection::label),
                );
            }
            WorkspaceTool::AddNode => {
                let id = self.model.add_node(press.model_x, press.model_y);
                self.selection = Some(Selection::Node(id));
                self.analysis = AnalysisState::Idle;
                self.set_status(StatusLevel::Success, format!("Node {id} added"));
            }
            WorkspaceTool::DrawMember => self.handle_member_press(press.target),
            WorkspaceTool::AssignLoad => self.handle_load_press(press.target),
            WorkspaceTool::AssignSupport => self.handle_support_press(press.target),
        }
    }

    fn add_node_from_tree(&mut self) {
        let id = self.model.add_default_node();
        self.selection = Some(Selection::Node(id));
        self.draft.clear();
        self.analysis = AnalysisState::Idle;
        self.set_status(StatusLevel::Success, format!("Node {id} added"));
    }

    fn add_member_from_tree(&mut self) {
        let result = if let Some(start) = self.draft.member_start {
            let next_node = self
                .model
                .nodes
                .iter()
                .find(|node| node.id != start)
                .map(|node| node.id);

            if let Some(node_id) = next_node {
                self.model.add_frame_member(start, node_id)
            } else {
                self.model.add_default_frame_member()
            }
        } else {
            self.model.add_default_frame_member()
        };

        match result {
            Ok(id) => {
                self.selection = Some(Selection::Element(id));
                self.draft.clear();
                self.analysis = AnalysisState::Idle;
                self.set_status(StatusLevel::Success, format!("Member {id} added"));
            }
            Err(error) => self.set_status(StatusLevel::Warning, error),
        }
    }

    fn add_load_from_tree(&mut self) {
        self.load_builder = match self.selection {
            Some(Selection::Node(node_id)) => Some(LoadBuilder::point(node_id)),
            Some(Selection::Element(element_id)) => Some(LoadBuilder::distributed(element_id)),
            None => None,
        };

        let Some(builder) = &self.load_builder else {
            self.set_status(
                StatusLevel::Warning,
                "Select a node or member before adding a load",
            );
            return;
        };

        self.draft.clear();
        self.set_status(
            StatusLevel::Neutral,
            format!("Define {} load", builder.kind.label()),
        );
    }

    fn apply_load(&mut self) {
        let Some(builder) = self.load_builder.clone() else {
            self.set_status(
                StatusLevel::Warning,
                "Select a node or member and use + before applying a load",
            );
            return;
        };

        let magnitude = match expression::evaluate(builder.magnitude.trim()) {
            Ok(value) => value * 1000.0,
            Err(error) => {
                self.set_status(StatusLevel::Warning, format!("Load magnitude: {error}"));
                return;
            }
        };

        match builder.target {
            LoadTarget::Node(node_id) => {
                match self.model.add_nodal_load(node_id, builder.dof, magnitude) {
                    Ok(index) => {
                        self.selection = Some(Selection::Node(node_id));
                        self.load_builder = None;
                        self.draft.clear();
                        self.analysis = AnalysisState::Idle;
                        self.set_status(
                            StatusLevel::Success,
                            format!("Point load {index} assigned to node {node_id}"),
                        );
                    }
                    Err(error) => self.set_status(StatusLevel::Warning, error),
                }
            }
            LoadTarget::Element(element_id) => {
                match self
                    .model
                    .add_distributed_load(element_id, builder.direction, magnitude)
                {
                    Ok(index) => {
                        self.selection = Some(Selection::Element(element_id));
                        self.load_builder = None;
                        self.draft.clear();
                        self.analysis = AnalysisState::Idle;
                        self.set_status(
                            StatusLevel::Success,
                            format!("Distributed load {index} assigned to member {element_id}"),
                        );
                    }
                    Err(error) => self.set_status(StatusLevel::Warning, error),
                }
            }
        }
    }

    fn add_support_from_tree(&mut self) {
        let Some(Selection::Node(node_id)) = self.selection else {
            self.set_status(
                StatusLevel::Warning,
                "Select one node before adding a support",
            );
            return;
        };

        self.support_builder = Some(SupportBuilder::new(node_id));
        self.draft.clear();
        self.set_status(
            StatusLevel::Neutral,
            format!("Choose a support type for node {node_id}"),
        );
    }

    fn add_support_preset(&mut self, preset: SupportPreset) {
        let Some(builder) = self.support_builder else {
            self.set_status(
                StatusLevel::Warning,
                "Select a node and use + before choosing a support",
            );
            return;
        };

        let result = self.model.assign_support_preset(builder.node_id, preset);
        match result {
            Ok(index) => {
                let node_id = self.model.supports[index].node_id;
                self.selection = Some(Selection::Node(node_id));
                self.draft.clear();
                self.support_builder = None;
                self.analysis = AnalysisState::Idle;
                self.set_status(
                    StatusLevel::Success,
                    format!("{} support assigned to node {node_id}", preset.label()),
                );
            }
            Err(error) => self.set_status(StatusLevel::Warning, error),
        }
    }

    fn toggle_custom_support_dof(&mut self, dof: Dof, restrained: bool) {
        let Some(builder) = &mut self.support_builder else {
            self.set_status(
                StatusLevel::Warning,
                "Select a node and use + before editing a custom support",
            );
            return;
        };

        match dof {
            Dof::Ux => builder.ux = restrained,
            Dof::Uy => builder.uy = restrained,
            Dof::Uz => builder.uz = restrained,
            Dof::Rx => builder.rx = restrained,
            Dof::Ry => builder.ry = restrained,
            Dof::Rz => builder.rz = restrained,
        }
    }

    fn apply_custom_support(&mut self) {
        let Some(builder) = self.support_builder else {
            self.set_status(
                StatusLevel::Warning,
                "Select a node and use + before applying a custom support",
            );
            return;
        };

        let dofs = support_builder_dofs(builder);
        match self.model.assign_custom_support(builder.node_id, &dofs) {
            Ok(index) => {
                let node_id = self.model.supports[index].node_id;
                self.selection = Some(Selection::Node(node_id));
                self.draft.clear();
                self.support_builder = None;
                self.analysis = AnalysisState::Idle;
                self.set_status(
                    StatusLevel::Success,
                    format!("Custom support assigned to node {node_id}"),
                );
            }
            Err(error) => self.set_status(StatusLevel::Warning, error),
        }
    }

    fn delete_node(&mut self, node_id: usize) {
        match self.model.remove_node(node_id) {
            Ok(()) => {
                self.selection = None;
                self.draft.clear();
                self.clear_edit_drafts();
                self.analysis = AnalysisState::Idle;
                self.set_status(StatusLevel::Success, format!("Node {node_id} deleted"));
            }
            Err(error) => self.set_status(StatusLevel::Warning, error),
        }
    }

    fn delete_member(&mut self, element_id: usize) {
        match self.model.remove_element(element_id) {
            Ok(()) => {
                if self.selection == Some(Selection::Element(element_id)) {
                    self.selection = None;
                }
                self.draft.clear();
                self.member_endpoint_edits
                    .retain(|(id, _), _| *id != element_id);
                self.analysis = AnalysisState::Idle;
                self.set_status(StatusLevel::Success, format!("Member {element_id} deleted"));
            }
            Err(error) => self.set_status(StatusLevel::Warning, error),
        }
    }

    fn delete_point_load(&mut self, index: usize) {
        match self.model.remove_nodal_load(index) {
            Ok(()) => {
                self.load_builder = None;
                self.draft.clear();
                self.analysis = AnalysisState::Idle;
                self.set_status(StatusLevel::Success, format!("Point load {index} deleted"));
            }
            Err(error) => self.set_status(StatusLevel::Warning, error),
        }
    }

    fn delete_distributed_load(&mut self, index: usize) {
        match self.model.remove_distributed_load(index) {
            Ok(()) => {
                self.load_builder = None;
                self.draft.clear();
                self.analysis = AnalysisState::Idle;
                self.set_status(
                    StatusLevel::Success,
                    format!("Distributed load {index} deleted"),
                );
            }
            Err(error) => self.set_status(StatusLevel::Warning, error),
        }
    }

    fn delete_support_group(&mut self, node_id: usize) {
        match self.model.remove_supports_at_node(node_id) {
            Ok(()) => {
                self.support_builder = None;
                self.draft.clear();
                self.analysis = AnalysisState::Idle;
                self.set_status(
                    StatusLevel::Success,
                    format!("Supports removed from node {node_id}"),
                );
            }
            Err(error) => self.set_status(StatusLevel::Warning, error),
        }
    }

    fn handle_node_coordinate_submit(&mut self, node_id: usize, axis: CoordinateAxis) {
        let key = (node_id, axis);
        let value = self.node_coordinate_edits.get(&key).cloned().or_else(|| {
            self.model
                .node(node_id)
                .map(|node| coordinate_text(node, axis))
        });

        let Some(value) = value else {
            self.set_status(
                StatusLevel::Warning,
                format!("Node {node_id} does not exist."),
            );
            return;
        };

        let trimmed = value.trim();

        if trimmed.is_empty() {
            self.set_status(
                StatusLevel::Warning,
                format!("Enter a value for node {node_id} {}", axis.label()),
            );
            return;
        }

        let coordinate = match expression::evaluate(trimmed) {
            Ok(value) => value,
            Err(error) => {
                self.set_status(
                    StatusLevel::Warning,
                    format!("Node {node_id} {}: {error}", axis.label()),
                );
                return;
            }
        };

        match self.model.update_node_coordinate(node_id, axis, coordinate) {
            Ok(()) => {
                self.node_coordinate_edits.remove(&key);
                self.selection = Some(Selection::Node(node_id));
                self.draft.clear();
                self.analysis = AnalysisState::Idle;
                self.set_status(
                    StatusLevel::Success,
                    format!("Node {node_id} {} set to {coordinate:.3}", axis.label()),
                );
            }
            Err(error) => self.set_status(StatusLevel::Warning, error),
        }
    }

    fn handle_member_endpoint_submit(&mut self, element_id: usize, endpoint: MemberEndpoint) {
        let key = (element_id, endpoint);
        let value = self.member_endpoint_edits.get(&key).cloned().or_else(|| {
            self.model.element(element_id).map(|element| {
                let (_, node_i, node_j) = crate::state::element_data(element);
                match endpoint {
                    MemberEndpoint::Start => node_i,
                    MemberEndpoint::End => node_j,
                }
                .to_string()
            })
        });

        let Some(value) = value else {
            self.set_status(
                StatusLevel::Warning,
                format!("Member {element_id} does not exist."),
            );
            return;
        };

        let node_id = match parse_usize(value.trim(), "node id") {
            Ok(value) => value,
            Err(error) => {
                self.set_status(StatusLevel::Warning, error);
                return;
            }
        };

        match self
            .model
            .update_member_endpoint(element_id, endpoint, node_id)
        {
            Ok(()) => {
                self.member_endpoint_edits.remove(&key);
                self.selection = Some(Selection::Element(element_id));
                self.draft.clear();
                self.analysis = AnalysisState::Idle;
                self.set_status(
                    StatusLevel::Success,
                    format!(
                        "Member {element_id} {} set to node {node_id}",
                        endpoint.label()
                    ),
                );
            }
            Err(error) => self.set_status(StatusLevel::Warning, error),
        }
    }

    fn handle_member_press(&mut self, target: Option<Selection>) {
        let Some(Selection::Node(node_id)) = target else {
            self.set_status(StatusLevel::Warning, "Select a node endpoint");
            return;
        };

        if let Some(start) = self.draft.member_start {
            match self.model.add_frame_member(start, node_id) {
                Ok(id) => {
                    self.selection = Some(Selection::Element(id));
                    self.draft.clear();
                    self.analysis = AnalysisState::Idle;
                    self.set_status(StatusLevel::Success, format!("Member {id} added"));
                }
                Err(error) => {
                    self.set_status(StatusLevel::Warning, error);
                }
            }
        } else {
            self.selection = Some(Selection::Node(node_id));
            self.draft.member_start = Some(node_id);
            self.set_status(
                StatusLevel::Neutral,
                format!("Member start: node {node_id}"),
            );
        }
    }

    fn handle_load_press(&mut self, target: Option<Selection>) {
        let Some(selection @ (Selection::Node(_) | Selection::Element(_))) = target else {
            self.set_status(StatusLevel::Warning, "Select a node or member for the load");
            return;
        };

        self.selection = Some(selection);
        self.draft.clear();
        self.set_status(
            StatusLevel::Neutral,
            "Selection ready. Use + in Loads to assign load",
        );
    }

    fn handle_support_press(&mut self, target: Option<Selection>) {
        let Some(Selection::Node(node_id)) = target else {
            self.set_status(StatusLevel::Warning, "Select a node for the support");
            return;
        };

        self.selection = Some(Selection::Node(node_id));
        self.draft.clear();
        self.set_status(
            StatusLevel::Neutral,
            format!("Node {node_id} selected. Use + in Supports to assign restraints"),
        );
    }

    fn solve(&mut self) {
        match run_basic_analysis(&self.model) {
            Ok(summary) => {
                let displacement = summary.max_displacement;
                self.result_display = ResultDisplay::Combined;
                self.analysis = AnalysisState::Success(summary);
                self.set_status(
                    StatusLevel::Success,
                    format!("Solved. Max displacement {displacement:.3e} m"),
                );
            }
            Err(error) => {
                self.analysis = AnalysisState::Failed(error.clone());
                self.set_status(StatusLevel::Error, error);
            }
        }
    }

    fn set_status(&mut self, level: StatusLevel, text: impl Into<String>) {
        self.status = StatusLine {
            text: text.into(),
            level,
        };
    }

    fn clear_edit_drafts(&mut self) {
        self.node_coordinate_edits.clear();
        self.member_endpoint_edits.clear();
        self.load_builder = None;
        self.support_builder = None;
    }
}

fn support_builder_dofs(builder: SupportBuilder) -> Vec<Dof> {
    [
        (builder.ux, Dof::Ux),
        (builder.uy, Dof::Uy),
        (builder.uz, Dof::Uz),
        (builder.rx, Dof::Rx),
        (builder.ry, Dof::Ry),
        (builder.rz, Dof::Rz),
    ]
    .into_iter()
    .filter_map(|(restrained, dof)| restrained.then_some(dof))
    .collect()
}

fn coordinate_text(node: &model::node::Node, axis: CoordinateAxis) -> String {
    match axis {
        CoordinateAxis::X => node.x,
        CoordinateAxis::Y => node.y,
        CoordinateAxis::Z => node.z,
    }
    .to_string()
}

fn parse_usize(value: &str, label: &str) -> Result<usize, String> {
    value.parse().map_err(|_| format!("Enter a valid {label}."))
}

impl StatusLine {
    fn neutral(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            level: StatusLevel::Neutral,
        }
    }
}
