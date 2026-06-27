use iced::widget::{button, column, container, row, slider, text};
use iced::{Alignment, Element, Fill, Length};

use crate::{
    app::Message,
    state::{AnalysisState, ResultDisplay, StructuralModel},
    theme,
};

pub fn view<'a>(
    model: &'a StructuralModel,
    analysis: &'a AnalysisState,
    result_display: ResultDisplay,
    result_scale: f64,
) -> Element<'a, Message> {
    column![
        panel_title("Analysis"),
        solve_panel(model, analysis),
        visualization_panel(analysis, result_display, result_scale),
    ]
    .spacing(16)
    .padding(14)
    .width(Fill)
    .into()
}

fn solve_panel(model: &StructuralModel, analysis: &AnalysisState) -> Element<'static, Message> {
    let status = match analysis {
        AnalysisState::Idle => "Not solved".to_string(),
        AnalysisState::Success(summary) => {
            format!("Solved  max |u| {:.3e} m", summary.max_displacement)
        }
        AnalysisState::Failed(error) => error.clone(),
    };

    panel(
        "Run",
        column![
            property("Members", model.elements.len().to_string()),
            property("Supports", model.supports.len().to_string()),
            property(
                "Loads",
                (model.nodal_loads.len() + model.distributed_loads.len()).to_string()
            ),
            text(status)
                .size(13)
                .color(status_color(analysis))
                .width(Fill),
            button(text("Solve").size(15))
                .padding([9, 18])
                .width(Fill)
                .style(theme::primary_button)
                .on_press(Message::SolveRequested),
        ]
        .spacing(8),
    )
}

fn visualization_panel(
    analysis: &AnalysisState,
    result_display: ResultDisplay,
    result_scale: f64,
) -> Element<'static, Message> {
    let solved = matches!(analysis, AnalysisState::Success(_));

    let modes = ResultDisplay::ALL
        .into_iter()
        .fold(column![].spacing(6), |column, display| {
            let mut control = button(
                row![
                    text(display.label()).size(13).width(Fill),
                    text(mode_state(display, result_display, solved))
                        .size(12)
                        .color(theme::TEXT_MUTED),
                ]
                .align_y(Alignment::Center),
            )
            .padding([7, 9])
            .width(Fill)
            .style(if display == result_display {
                theme::tool_button_active
            } else {
                theme::secondary_button
            });

            if solved || !display.needs_results() {
                control = control.on_press(Message::ResultDisplaySelected(display));
            }

            column.push(control)
        });

    panel(
        "Visualization",
        column![
            modes,
            row![
                text("Scale")
                    .size(13)
                    .color(theme::TEXT_MUTED)
                    .width(Length::Fixed(58.0)),
                slider(0.0..=400.0, result_scale, Message::ResultScaleChanged)
                    .step(10.0)
                    .width(Fill),
                text(format!("{result_scale:.0}"))
                    .size(13)
                    .color(theme::TEXT)
                    .width(Length::Fixed(54.0)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(12),
    )
}

fn panel<'a>(
    title: &'static str,
    content: iced::widget::Column<'a, Message>,
) -> Element<'a, Message> {
    container(column![text(title).size(15).color(theme::TEXT), content].spacing(10))
        .padding(10)
        .width(Fill)
        .style(theme::inset)
        .into()
}

fn panel_title(label: &str) -> Element<'_, Message> {
    text(label).size(18).color(theme::TEXT).into()
}

fn property(label: &'static str, value: String) -> Element<'static, Message> {
    row![
        text(label)
            .size(13)
            .color(theme::TEXT_MUTED)
            .width(Length::Fixed(74.0)),
        text(value).size(14).color(theme::TEXT).width(Fill),
    ]
    .spacing(8)
    .into()
}

fn mode_state(display: ResultDisplay, active: ResultDisplay, solved: bool) -> &'static str {
    if display == active {
        "Active"
    } else if display.needs_results() && !solved {
        "Solve"
    } else {
        ""
    }
}

fn status_color(analysis: &AnalysisState) -> iced::Color {
    match analysis {
        AnalysisState::Idle => theme::TEXT_MUTED,
        AnalysisState::Success(_) => theme::SUPPORT,
        AnalysisState::Failed(_) => theme::LOAD,
    }
}
