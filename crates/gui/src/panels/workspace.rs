use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Fill, Length};

use crate::{
    app::Message,
    renderer::viewport_canvas::ViewportCanvas,
    state::{
        AnalysisState, InteractionDraft, ResultDisplay, Selection, StructuralModel, WorkspaceTool,
    },
    theme,
    viewport::ViewportState,
};

pub fn view<'a>(
    model: &'a StructuralModel,
    selection: Option<Selection>,
    tool: WorkspaceTool,
    draft: InteractionDraft,
    viewport: ViewportState,
    analysis: &'a AnalysisState,
    result_display: ResultDisplay,
    result_scale: f64,
) -> Element<'a, Message> {
    let canvas = ViewportCanvas::new(
        model,
        selection,
        tool,
        draft,
        viewport,
        analysis,
        result_display,
        result_scale,
    )
    .view()
    .map(|event| match event {
        crate::viewport::ViewportEvent::Pressed(press) => Message::ViewportPressed(press),
        crate::viewport::ViewportEvent::Changed(update) => Message::ViewportChanged(update),
    });

    container(
        column![
            header(
                tool,
                selection,
                viewport,
                analysis,
                result_display,
                result_scale,
            ),
            container(canvas)
                .width(Fill)
                .height(Fill)
                .style(theme::viewport),
        ]
        .spacing(8),
    )
    .width(Fill)
    .height(Fill)
    .into()
}

fn header(
    tool: WorkspaceTool,
    selection: Option<Selection>,
    viewport: ViewportState,
    analysis: &AnalysisState,
    result_display: ResultDisplay,
    result_scale: f64,
) -> Element<'static, Message> {
    let selection = selection.map_or_else(|| "None".to_string(), Selection::label);
    let solved = matches!(analysis, AnalysisState::Success(_));
    let result_label = if solved {
        result_display.label().to_string()
    } else {
        "Unsolved".to_string()
    };

    let modes = ResultDisplay::ALL
        .into_iter()
        .fold(row![].spacing(4), |row, display| {
            let mut control = button(text(display.label()).size(12))
                .padding([6, 9])
                .style(if display == result_display {
                    theme::tool_button_active
                } else {
                    theme::secondary_button
                });

            if solved || !display.needs_results() {
                control = control.on_press(Message::ResultDisplaySelected(display));
            }

            row.push(control)
        });

    let scale_controls = row![
        button(text("-").size(14))
            .padding([6, 10])
            .style(theme::secondary_button)
            .on_press(Message::ResultScaleChanged(result_scale - 20.0)),
        text(format!("{result_scale:.0} px"))
            .size(13)
            .color(theme::TEXT_MUTED),
        button(text("+").size(14))
            .padding([6, 10])
            .style(theme::secondary_button)
            .on_press(Message::ResultScaleChanged(result_scale + 20.0)),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    container(
        row![
            text("Workspace").size(15).color(theme::TEXT),
            text(tool.label()).size(14).color(theme::TEXT_MUTED),
            text(selection).size(14).color(theme::TEXT_MUTED),
            text(result_label).size(14).color(theme::TEXT_MUTED),
            modes,
            scale_controls,
            text(format!("{:.0}%", viewport.zoom / 58.0 * 100.0))
                .size(14)
                .color(theme::TEXT_MUTED)
                .width(Length::Shrink),
        ]
        .spacing(16)
        .align_y(Alignment::Center),
    )
    .padding([9, 12])
    .width(Fill)
    .style(theme::status_bar)
    .into()
}
