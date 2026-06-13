use iced::mouse;
use iced::widget::canvas;
use iced::{
    Color, Element, Font, Length, Pixels, Point, Rectangle, Renderer, Theme, Vector, alignment,
};
use model::{elements::StructuralElement, node::Node};

use crate::{
    state::{Selection, StructuralModel},
    theme,
    viewport::{ViewportEvent, ViewportState, ViewportUpdate},
};

#[derive(Debug, Clone, Copy)]
pub struct ViewportCanvas<'a> {
    model: &'a StructuralModel,
    selection: Option<Selection>,
    viewport: ViewportState,
}

#[derive(Debug, Default)]
pub struct CanvasState {
    drag_origin: Option<Point>,
}

impl<'a> ViewportCanvas<'a> {
    pub fn new(
        model: &'a StructuralModel,
        selection: Option<Selection>,
        viewport: ViewportState,
    ) -> Self {
        Self {
            model,
            selection,
            viewport,
        }
    }

    pub fn view(self) -> Element<'a, ViewportEvent> {
        canvas(self).width(Length::Fill).height(Length::Fill).into()
    }

    fn node_screen_position(&self, node: &Node, bounds: Rectangle) -> Point {
        let origin = Point::new(
            bounds.width * 0.5 + self.viewport.pan_x,
            bounds.height * 0.64 + self.viewport.pan_y,
        );

        Point::new(
            origin.x + node.x as f32 * self.viewport.zoom,
            origin.y - node.y as f32 * self.viewport.zoom,
        )
    }

    fn hit_test(&self, cursor: Point, bounds: Rectangle) -> Option<Selection> {
        let node_hit = self.model.nodes.iter().find_map(|node| {
            let point = self.node_screen_position(node, bounds);
            (distance(cursor, point) <= 10.0).then_some(Selection::Node(node.id))
        });

        if node_hit.is_some() {
            return node_hit;
        }

        self.model.elements.iter().find_map(|element| {
            let (id, node_i, node_j) = element_data(element);
            let ni = self.model.nodes.iter().find(|node| node.id == node_i)?;
            let nj = self.model.nodes.iter().find(|node| node.id == node_j)?;
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
        let Some(position) = cursor.position_in(bounds) else {
            state.drag_origin = None;
            return None;
        };

        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) => {
                state.drag_origin = Some(position);
                Some(canvas::Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) => {
                state.drag_origin = None;
                Some(canvas::Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(origin) = state.drag_origin.replace(position) {
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
                        pivot_x: position.x,
                        pivot_y: position.y,
                    }))
                    .and_capture(),
                )
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => Some(
                canvas::Action::publish(ViewportEvent::Selected(self.hit_test(position, bounds)))
                    .and_capture(),
            ),
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

        draw_grid(&mut frame, bounds);
        draw_axes(&mut frame, bounds, self.viewport);
        self.draw_elements(&mut frame, bounds);
        self.draw_loads(&mut frame, bounds);
        self.draw_supports(&mut frame, bounds);
        self.draw_nodes(&mut frame, bounds);

        if self.model.nodes.is_empty() {
            frame.fill_text(canvas::Text {
                content: "Start by adding nodes to define the structure.".to_string(),
                position: Point::new(bounds.width * 0.5, bounds.height * 0.5),
                color: theme::TEXT_MUTED,
                size: Pixels::from(16.0),
                line_height: iced::widget::text::LineHeight::default(),
                font: Font::DEFAULT,
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Center,
                shaping: iced::widget::text::Shaping::Basic,
                max_width: bounds.width - 48.0,
            });
        }

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.drag_origin.is_some() {
            return mouse::Interaction::Grabbing;
        }

        if cursor.position_in(bounds).is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl ViewportCanvas<'_> {
    fn draw_elements(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        for element in &self.model.elements {
            let (id, node_i, node_j) = element_data(element);
            let Some(ni) = self.model.nodes.iter().find(|node| node.id == node_i) else {
                continue;
            };
            let Some(nj) = self.model.nodes.iter().find(|node| node.id == node_j) else {
                continue;
            };

            let a = self.node_screen_position(ni, bounds);
            let b = self.node_screen_position(nj, bounds);
            let selected = self.selection == Some(Selection::Element(id));
            let path = canvas::Path::line(a, b);

            frame.stroke(
                &path,
                canvas::Stroke {
                    style: canvas::Style::Solid(if selected {
                        theme::ACCENT
                    } else {
                        Color::from_rgb(0.118, 0.161, 0.216)
                    }),
                    width: if selected { 4.0 } else { 2.5 },
                    ..canvas::Stroke::default()
                },
            );

            let midpoint = Point::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
            frame.fill_text(canvas::Text {
                content: id.to_string(),
                position: midpoint + Vector::new(0.0, -10.0),
                color: theme::TEXT_MUTED,
                size: Pixels::from(12.0),
                line_height: iced::widget::text::LineHeight::default(),
                font: Font::DEFAULT,
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Center,
                shaping: iced::widget::text::Shaping::Basic,
                max_width: 48.0,
            });
        }
    }

    fn draw_nodes(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        for node in &self.model.nodes {
            let point = self.node_screen_position(node, bounds);
            let selected = self.selection == Some(Selection::Node(node.id));
            let marker = canvas::Path::circle(point, if selected { 7.0 } else { 5.0 });

            frame.fill(
                &marker,
                if selected {
                    theme::ACCENT
                } else {
                    Color::from_rgb(0.050, 0.090, 0.130)
                },
            );

            frame.fill_text(canvas::Text {
                content: format!("N{}", node.id),
                position: point + Vector::new(0.0, 16.0),
                color: theme::TEXT,
                size: Pixels::from(12.0),
                line_height: iced::widget::text::LineHeight::default(),
                font: Font::DEFAULT,
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Center,
                shaping: iced::widget::text::Shaping::Basic,
                max_width: 56.0,
            });
        }
    }

    fn draw_supports(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        for support in &self.model.supports {
            let Some(node) = self
                .model
                .nodes
                .iter()
                .find(|node| node.id == support.node_id)
            else {
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
                    style: canvas::Style::Solid(theme::SUCCESS),
                    width: 1.5,
                    ..canvas::Stroke::default()
                },
            );
        }
    }

    fn draw_loads(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        for load in &self.model.nodal_loads {
            let Some(node) = self.model.nodes.iter().find(|node| node.id == load.node_id) else {
                continue;
            };

            let point = self.node_screen_position(node, bounds);
            let direction = if load.magnitude < 0.0 { 1.0 } else { -1.0 };
            let start = point + Vector::new(0.0, -direction * 42.0);
            let end = point + Vector::new(0.0, -direction * 12.0);

            let shaft = canvas::Path::line(start, end);
            frame.stroke(
                &shaft,
                canvas::Stroke {
                    style: canvas::Style::Solid(theme::DANGER),
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
            frame.fill(&head, theme::DANGER);
        }
    }
}

fn draw_grid(frame: &mut canvas::Frame, bounds: Rectangle) {
    let spacing = 48.0;
    let color = Color::from_rgba(0.690, 0.733, 0.773, 0.30);

    let mut x = 0.0;
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

    let mut y = 0.0;
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
    let origin = Point::new(
        bounds.width * 0.5 + viewport.pan_x,
        bounds.height * 0.64 + viewport.pan_y,
    );

    frame.stroke(
        &canvas::Path::line(
            Point::new(0.0, origin.y),
            Point::new(bounds.width, origin.y),
        ),
        canvas::Stroke {
            style: canvas::Style::Solid(Color::from_rgba(0.200, 0.260, 0.320, 0.34)),
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
            style: canvas::Style::Solid(Color::from_rgba(0.200, 0.260, 0.320, 0.34)),
            width: 1.0,
            ..canvas::Stroke::default()
        },
    );
}

fn element_data(element: &StructuralElement) -> (usize, usize, usize) {
    match element {
        StructuralElement::Truss2D(element) => (element.id, element.node_i, element.node_j),
        StructuralElement::Truss3D(element) => (element.id, element.node_i, element.node_j),
        StructuralElement::Beam2D(element) => (element.id, element.node_i, element.node_j),
        StructuralElement::Beam3D(element) => (element.id, element.node_i, element.node_j),
        StructuralElement::Frame2D(element) => (element.id, element.node_i, element.node_j),
        StructuralElement::Frame3D(element) => (element.id, element.node_i, element.node_j),
    }
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
