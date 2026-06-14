use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Fill, Length, Task};

use crate::{
    panels,
    state::{
        AnalysisState, InteractionDraft, Selection, StructuralModel, WorkspaceTool,
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
    pub status: StatusLine,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToolSelected(WorkspaceTool),
    SelectionRequested(Option<Selection>),
    ViewportPressed(ViewportPress),
    ViewportChanged(ViewportUpdate),
    SolveRequested,
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
            status: StatusLine::neutral("Ready"),
        }
    }
}

impl BajrangApp {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToolSelected(tool) => {
                self.tool = tool;
                self.draft.clear();
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
            Message::ViewportPressed(press) => self.handle_viewport_press(press),
            Message::ViewportChanged(update) => self.viewport.apply(update),
            Message::SolveRequested => self.solve(),
            Message::FitView => {
                self.viewport = ViewportState::default();
                self.set_status(StatusLevel::Neutral, "View reset");
            }
            Message::NewModel => {
                self.model = StructuralModel::empty();
                self.selection = None;
                self.draft.clear();
                self.analysis = AnalysisState::Idle;
                self.set_status(StatusLevel::Neutral, "New model");
            }
            Message::LoadSample => {
                *self = Self::default();
            }
        }

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let sidebar = container(scrollable(panels::model_tree::view(
            &self.model,
            self.selection,
            self.draft,
        )))
        .width(292)
        .height(Fill)
        .style(theme::panel);

        let workspace = panels::workspace::view(
            &self.model,
            self.selection,
            self.tool,
            self.draft,
            self.viewport,
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
                    button(text("Solve").size(15))
                        .padding([9, 18])
                        .style(theme::primary_button)
                        .on_press(Message::SolveRequested),
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
                    self.model.nodal_loads.len()
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
            WorkspaceTool::Select => {
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
        let Some(Selection::Node(node_id)) = target else {
            self.set_status(StatusLevel::Warning, "Select a node for the load");
            return;
        };

        self.model.add_default_load(node_id);
        self.selection = Some(Selection::Node(node_id));
        self.analysis = AnalysisState::Idle;
        self.set_status(
            StatusLevel::Success,
            format!("Load assigned to node {node_id}"),
        );
    }

    fn handle_support_press(&mut self, target: Option<Selection>) {
        let Some(Selection::Node(node_id)) = target else {
            self.set_status(StatusLevel::Warning, "Select a node for the support");
            return;
        };

        self.model.assign_pin_support(node_id);
        self.selection = Some(Selection::Node(node_id));
        self.analysis = AnalysisState::Idle;
        self.set_status(
            StatusLevel::Success,
            format!("Pin support assigned to node {node_id}"),
        );
    }

    fn solve(&mut self) {
        match run_basic_analysis(&self.model) {
            Ok(summary) => {
                let displacement = summary.max_displacement;
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
}

impl StatusLine {
    fn neutral(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            level: StatusLevel::Neutral,
        }
    }
}
