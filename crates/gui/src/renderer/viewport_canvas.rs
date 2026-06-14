use iced::mouse;
use iced::widget::canvas;
use iced::{
    Color, Element, Font, Length, Pixels, Point, Rectangle, Renderer, Theme, Vector, alignment,
};
use model::node::Node;

use crate::{
    state::{
        InteractionDraft, Selection, StructuralModel, WorkspaceTool, element_data, element_kind,
    },
    theme,
    viewport::{ViewportEvent, ViewportPress, ViewportState, ViewportUpdate},
};

#[derive(Debug, Clone, Copy)]
pub struct ViewportCanvas<'a> {
    model: &'a StructuralModel,
    selection: Option<Selection>,
    tool: WorkspaceTool,
    draft: InteractionDraft,
    viewport: ViewportState,
}

#[derive(Debug, Default)]
pub struct CanvasState {
    pan_origin: Option<Point>,
}

impl<'a> ViewportCanvas<'a> {
    pub fn new(
        model: &'a StructuralModel,
        selection: Option<Selection>,
        tool: WorkspaceTool,
        draft: InteractionDraft,
        viewport: ViewportState,
    ) -> Self {
        Self {
            model,
            selection,
            tool,
            draft,
            viewport,
        }
    }

    pub fn view(self) -> Element<'a, ViewportEvent> {
        canvas(self).width(Length::Fill).height(Length::Fill).into()
    }

    fn node_screen_position(&self, node: &Node, bounds: Rectangle) -> Point {
        self.viewport.model_to_screen(node.x, node.y, bounds)
    }

    fn hit_test(&self, cursor: Point, bounds: Rectangle) -> Option<Selection> {
        if let Some(node) = self.model.nodes.iter().find(|node| {
            let point = self.node_screen_position(node, bounds);
            distance(cursor, point) <= 10.0
        }) {
            return Some(Selection::Node(node.id));
        }

        self.model.elements.iter().find_map(|element| {
            let (id, node_i, node_j) = element_data(element);
            let ni = self.model.node(node_i)?;
            let nj = self.model.node(node_j)?;
            let a = self.node_screen_position(ni, bounds);
            let b = self.node_screen_position(nj, bounds);

            (distance_to_segment(cursor, a, b) <= 8.0).then_some(Selection::Element(id))
        })
    }
}

impl canvas::Program<ViewportEvent> for ViewportCanvas<'_> {
    type State = CanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<ViewportEvent>> {
        match event {
            canvas::Event::Mouse(mouse::Event::ButtonReleased(_)) => {
                state.pan_origin = None;
                return Some(canvas::Action::capture());
            }
            canvas::Event::Mouse(mouse::Event::CursorLeft) => {
                state.pan_origin = None;
                return None;
            }
            canvas::Event::Mouse(mouse::Event::CursorEntered) => {
                state.pan_origin = None;
                return None;
            }
            _ => {}
        }

        let Some(position) = cursor.position_in(bounds) else {
            state.pan_origin = None;
            return None;
        };

        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
                state.pan_origin = Some(position);
                Some(canvas::Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(origin) = state.pan_origin {
                    state.pan_origin = Some(position);
                    let delta = position - origin;
                    return Some(
                        canvas::Action::publish(ViewportEvent::Changed(ViewportUpdate::Pan {
                            dx: delta.x,
                            dy: delta.y,
                        }))
                        .and_capture(),
                    );
                }

                None
            }
            canvas::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let scroll_y = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y,
                    mouse::ScrollDelta::Pixels { y, .. } => *y / 40.0,
                };
                let factor = if scroll_y > 0.0 { 1.12 } else { 0.89 };

                Some(
                    canvas::Action::publish(ViewportEvent::Changed(ViewportUpdate::Zoom {
                        factor,
                        pivot: position,
                    }))
                    .and_capture(),
                )
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.pan_origin = None;
                let (model_x, model_y) = self.viewport.screen_to_model(position, bounds);

                Some(
                    canvas::Action::publish(ViewportEvent::Pressed(ViewportPress {
                        target: self.hit_test(position, bounds),
                        model_x,
                        model_y,
                    }))
                    .and_capture(),
                )
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(_)) => {
                state.pan_origin = None;
                None
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        draw_grid(&mut frame, bounds, self.viewport);
        draw_axes(&mut frame, bounds, self.viewport);
        self.draw_members(&mut frame, bounds);
        self.draw_loads(&mut frame, bounds);
        self.draw_supports(&mut frame, bounds);
        self.draw_nodes(&mut frame, bounds);
        self.draw_empty_state(&mut frame, bounds);

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.position_in(bounds).is_some() {
            match self.tool {
                WorkspaceTool::Select => mouse::Interaction::Idle,
                _ => mouse::Interaction::Crosshair,
            }
        } else {
            mouse::Interaction::default()
        }
    }
}

impl ViewportCanvas<'_> {
    fn draw_members(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        for element in &self.model.elements {
            let (id, node_i, node_j) = element_data(element);
            let Some(ni) = self.model.node(node_i) else {
                continue;
            };
            let Some(nj) = self.model.node(node_j) else {
                continue;
            };

            let a = self.node_screen_position(ni, bounds);
            let b = self.node_screen_position(nj, bounds);
            let selected = self.selection == Some(Selection::Element(id));

            frame.stroke(
                &canvas::Path::line(a, b),
                canvas::Stroke {
                    style: canvas::Style::Solid(if selected {
                        theme::ACCENT
                    } else {
                        Color::from_rgb(0.812, 0.847, 0.867)
                    }),
                    width: if selected { 4.0 } else { 2.5 },
                    ..canvas::Stroke::default()
                },
            );

            let midpoint = Point::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
            label(
                frame,
                format!("{} {id}", element_kind(element)),
                midpoint + Vector::new(0.0, -12.0),
                theme::TEXT_MUTED,
                11.0,
                86.0,
            );
        }
    }

    fn draw_nodes(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        for node in &self.model.nodes {
            let point = self.node_screen_position(node, bounds);
            let selected = self.selection == Some(Selection::Node(node.id));
            let draft = self.draft.member_start == Some(node.id);

            frame.fill(
                &canvas::Path::circle(point, if selected || draft { 7.0 } else { 5.0 }),
                if selected || draft {
                    theme::ACCENT
                } else {
                    Color::from_rgb(0.902, 0.925, 0.941)
                },
            );

            label(
                frame,
                format!("N{}", node.id),
                point + Vector::new(0.0, 17.0),
                theme::TEXT,
                12.0,
                44.0,
            );
        }
    }

    fn draw_supports(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        for support in &self.model.supports {
            let Some(node) = self.model.node(support.node_id) else {
                continue;
            };

            let point = self.node_screen_position(node, bounds);
            let path = canvas::Path::new(|builder| {
                builder.move_to(Point::new(point.x, point.y + 8.0));
                builder.line_to(Point::new(point.x - 9.0, point.y + 23.0));
                builder.line_to(Point::new(point.x + 9.0, point.y + 23.0));
                builder.close();
            });

            frame.stroke(
                &path,
                canvas::Stroke {
                    style: canvas::Style::Solid(theme::SUPPORT),
                    width: 1.5,
                    ..canvas::Stroke::default()
                },
            );
        }
    }

    fn draw_loads(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        for load in &self.model.nodal_loads {
            let Some(node) = self.model.node(load.node_id) else {
                continue;
            };

            let point = self.node_screen_position(node, bounds);
            let direction = if load.magnitude < 0.0 { 1.0 } else { -1.0 };
            let start = point + Vector::new(0.0, -direction * 44.0);
            let end = point + Vector::new(0.0, -direction * 13.0);

            frame.stroke(
                &canvas::Path::line(start, end),
                canvas::Stroke {
                    style: canvas::Style::Solid(theme::LOAD),
                    width: 2.0,
                    ..canvas::Stroke::default()
                },
            );

            let head = canvas::Path::new(|builder| {
                builder.move_to(end);
                builder.line_to(end + Vector::new(-5.0, -direction * 8.0));
                builder.line_to(end + Vector::new(5.0, -direction * 8.0));
                builder.close();
            });

            frame.fill(&head, theme::LOAD);
        }
    }

    fn draw_empty_state(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        if !self.model.nodes.is_empty() {
            return;
        }

        label(
            frame,
            "Empty model".to_string(),
            Point::new(bounds.width * 0.5, bounds.height * 0.5),
            theme::TEXT_MUTED,
            16.0,
            bounds.width - 48.0,
        );
    }
}

fn draw_grid(frame: &mut canvas::Frame, bounds: Rectangle, viewport: ViewportState) {
    let spacing = viewport.zoom.max(24.0);
    let origin = viewport.origin(bounds);
    let color = Color::from_rgba(0.459, 0.506, 0.545, 0.24);

    let mut x = origin.x.rem_euclid(spacing);
    while x <= bounds.width {
        frame.stroke(
            &canvas::Path::line(Point::new(x, 0.0), Point::new(x, bounds.height)),
            canvas::Stroke {
                style: canvas::Style::Solid(color),
                width: 1.0,
                ..canvas::Stroke::default()
            },
        );
        x += spacing;
    }

    let mut y = origin.y.rem_euclid(spacing);
    while y <= bounds.height {
        frame.stroke(
            &canvas::Path::line(Point::new(0.0, y), Point::new(bounds.width, y)),
            canvas::Stroke {
                style: canvas::Style::Solid(color),
                width: 1.0,
                ..canvas::Stroke::default()
            },
        );
        y += spacing;
    }
}

fn draw_axes(frame: &mut canvas::Frame, bounds: Rectangle, viewport: ViewportState) {
    let origin = viewport.origin(bounds);
    let color = Color::from_rgba(0.765, 0.800, 0.827, 0.34);

    frame.stroke(
        &canvas::Path::line(
            Point::new(0.0, origin.y),
            Point::new(bounds.width, origin.y),
        ),
        canvas::Stroke {
            style: canvas::Style::Solid(color),
            width: 1.0,
            ..canvas::Stroke::default()
        },
    );

    frame.stroke(
        &canvas::Path::line(
            Point::new(origin.x, 0.0),
            Point::new(origin.x, bounds.height),
        ),
        canvas::Stroke {
            style: canvas::Style::Solid(color),
            width: 1.0,
            ..canvas::Stroke::default()
        },
    );
}

fn label(
    frame: &mut canvas::Frame,
    content: String,
    position: Point,
    color: Color,
    size: f32,
    max_width: f32,
) {
    frame.fill_text(canvas::Text {
        content,
        position,
        color,
        size: Pixels::from(size),
        line_height: iced::widget::text::LineHeight::default(),
        font: Font::DEFAULT,
        align_x: alignment::Horizontal::Center.into(),
        align_y: alignment::Vertical::Center,
        shaping: iced::widget::text::Shaping::Basic,
        max_width,
    });
}

fn distance(a: Point, b: Point) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;

    (dx * dx + dy * dy).sqrt()
}

fn distance_to_segment(point: Point, a: Point, b: Point) -> f32 {
    let ab = b - a;
    let ap = point - a;
    let length_squared = ab.x * ab.x + ab.y * ab.y;

    if length_squared <= f32::EPSILON {
        return distance(point, a);
    }

    let t = ((ap.x * ab.x + ap.y * ab.y) / length_squared).clamp(0.0, 1.0);
    let projection = Point::new(a.x + ab.x * t, a.y + ab.y * t);

    distance(point, projection)
}
