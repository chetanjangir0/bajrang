use iced::widget::{column, container, row, text};
use iced::{Alignment, Element, Fill, Length};

use crate::{
    app::Message,
    renderer::viewport_canvas::ViewportCanvas,
    state::{InteractionDraft, Selection, StructuralModel, WorkspaceTool},
    theme,
    viewport::ViewportState,
};

pub fn view(
    model: &StructuralModel,
    selection: Option<Selection>,
    tool: WorkspaceTool,
    draft: InteractionDraft,
    viewport: ViewportState,
) -> Element<'_, Message> {
    let canvas = ViewportCanvas::new(model, selection, tool, draft, viewport)
        .view()
        .map(|event| match event {
            crate::viewport::ViewportEvent::Pressed(press) => Message::ViewportPressed(press),
            crate::viewport::ViewportEvent::Changed(update) => Message::ViewportChanged(update),
        });

    container(
        column![
            header(tool, selection, viewport),
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
) -> Element<'static, Message> {
    let selection = selection.map_or_else(|| "None".to_string(), Selection::label);

    container(
        row![
            text("Workspace").size(15).color(theme::TEXT),
            text(tool.label()).size(14).color(theme::TEXT_MUTED),
            text(selection).size(14).color(theme::TEXT_MUTED),
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
