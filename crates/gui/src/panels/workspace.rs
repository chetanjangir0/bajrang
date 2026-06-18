use iced::widget::{column, container, row, text};
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
            header(tool, selection, viewport, analysis, result_display),
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
) -> Element<'static, Message> {
    let selection = selection.map_or_else(|| "None".to_string(), Selection::label);
    let solved = matches!(analysis, AnalysisState::Success(_));
    let result_label = if solved {
        result_display.label().to_string()
    } else {
        "Unsolved".to_string()
    };

    container(
        row![
            text("Workspace").size(15).color(theme::TEXT),
            text(tool.label()).size(14).color(theme::TEXT_MUTED),
            text(selection).size(14).color(theme::TEXT_MUTED),
            text(result_label).size(14).color(theme::TEXT_MUTED),
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
