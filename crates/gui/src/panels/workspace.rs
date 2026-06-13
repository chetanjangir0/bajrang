use iced::widget::{column, container, text};
use iced::{Element, Fill};

use crate::{
    app::Message,
    renderer::viewport_canvas::ViewportCanvas,
    state::{Selection, StructuralModel, WorkspaceTool},
    theme,
    viewport::ViewportState,
};

pub fn view(
    model: &StructuralModel,
    selection: Option<Selection>,
    tool: WorkspaceTool,
    viewport: ViewportState,
) -> Element<'_, Message> {
    let canvas = ViewportCanvas::new(model, selection, viewport)
        .view()
        .map(|event| match event {
            crate::viewport::ViewportEvent::Selected(selection) => {
                Message::SelectionChanged(selection)
            }
            crate::viewport::ViewportEvent::Changed(update) => Message::ViewportChanged(update),
        });

    container(
        column![
            container(
                column![
                    text("Workspace").size(15).color(theme::TEXT),
                    text(format!(
                        "{} mode | wheel zoom, drag middle button to pan, click to select",
                        tool.label()
                    ))
                    .size(13)
                    .color(theme::TEXT_MUTED),
                ]
                .spacing(2)
            )
            .padding([10, 12])
            .width(Fill)
            .style(theme::status),
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
