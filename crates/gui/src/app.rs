use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Fill, Length, Task};

use crate::{
    panels,
    state::{AnalysisState, Selection, StructuralModel, run_basic_analysis},
    theme,
    tools::WorkspaceTool,
    viewport::{ViewportState, ViewportUpdate},
};

#[derive(Debug)]
pub struct BajrangApp {
    pub model: StructuralModel,
    pub tool: WorkspaceTool,
    pub selection: Option<Selection>,
    pub viewport: ViewportState,
    pub analysis: AnalysisState,
    pub status: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToolSelected(WorkspaceTool),
    SelectionChanged(Option<Selection>),
    ViewportChanged(ViewportUpdate),
    SolveRequested,
    FitView,
    ResetSample,
}

impl Default for BajrangApp {
    fn default() -> Self {
        Self {
            model: StructuralModel::sample(),
            tool: WorkspaceTool::Select,
            selection: None,
            viewport: ViewportState::default(),
            analysis: AnalysisState::NotRun,
            status: "Ready".to_string(),
        }
    }
}

impl BajrangApp {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToolSelected(tool) => {
                self.tool = tool;
                self.status = format!("{} tool active", tool.label());
            }
            Message::SelectionChanged(selection) => {
                self.selection = selection;
                self.status = self.selection.map_or_else(
                    || "Selection cleared".to_string(),
                    |selection| selection.label(),
                );
            }
            Message::ViewportChanged(update) => {
                self.viewport.apply(update);
            }
            Message::SolveRequested => match run_basic_analysis(&self.model) {
                Ok(summary) => {
                    let displacement = summary.max_displacement;
                    self.analysis = AnalysisState::Success(summary);
                    self.status = format!("Solved. Max |u| = {displacement:.3e} m");
                }
                Err(error) => {
                    self.analysis = AnalysisState::Failed(error.clone());
                    self.status = format!("Analysis failed: {error}");
                }
            },
            Message::FitView => {
                self.viewport = ViewportState::default();
                self.status = "Viewport fit to model extents".to_string();
            }
            Message::ResetSample => {
                *self = Self::default();
            }
        }

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let toolbar = self.toolbar();
        let left_panel = panels::model_tree::view(&self.model, self.selection);
        let center = panels::workspace::view(&self.model, self.selection, self.tool, self.viewport);
        let right_panel = panels::properties::view(&self.model, self.selection, &self.analysis);

        let body = row![
            container(scrollable(left_panel).height(Fill))
                .width(280)
                .height(Fill)
                .style(theme::panel),
            center,
            container(scrollable(right_panel).height(Fill))
                .width(320)
                .height(Fill)
                .style(theme::panel),
        ]
        .spacing(8)
        .height(Fill);

        let status = container(
            row![
                text(&self.status).size(14).color(theme::TEXT_MUTED),
                text(format!(
                    "{} nodes | {} elements | {} supports | {} loads",
                    self.model.nodes.len(),
                    self.model.elements.len(),
                    self.model.supports.len(),
                    self.model.nodal_loads.len()
                ))
                .size(14)
                .color(theme::TEXT_MUTED),
            ]
            .spacing(16)
            .align_y(Alignment::Center),
        )
        .padding([8, 12])
        .width(Fill)
        .style(theme::status);

        container(column![toolbar, body, status].spacing(8))
            .padding(8)
            .width(Fill)
            .height(Fill)
            .style(theme::app_background)
            .into()
    }

    fn toolbar(&self) -> Element<'_, Message> {
        let tool_buttons = WorkspaceTool::ALL.into_iter().fold(
            row![].spacing(6).align_y(Alignment::Center),
            |row, tool| {
                let label = if tool == self.tool {
                    format!("{} active", tool.label())
                } else {
                    tool.label().to_string()
                };

                row.push(
                    button(text(label).size(14))
                        .padding([8, 12])
                        .style(if tool == self.tool {
                            theme::primary_button
                        } else {
                            theme::secondary_button
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
                .spacing(2),
                tool_buttons,
                row![
                    button(text("Fit").size(14))
                        .padding([8, 12])
                        .style(theme::secondary_button)
                        .on_press(Message::FitView),
                    button(text("Reset").size(14))
                        .padding([8, 12])
                        .style(theme::secondary_button)
                        .on_press(Message::ResetSample),
                    button(text("Solve").size(14))
                        .padding([8, 16])
                        .style(theme::primary_button)
                        .on_press(Message::SolveRequested),
                ]
                .spacing(6)
            ]
            .spacing(24)
            .align_y(Alignment::Center),
        )
        .padding([10, 12])
        .width(Length::Fill)
        .style(theme::panel)
        .into()
    }
}
