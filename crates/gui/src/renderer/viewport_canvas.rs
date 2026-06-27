use bajrang_core::post::diagrams::DiagramKind;
use iced::mouse;
use iced::widget::canvas;
use iced::{
    Color, Element, Font, Length, Pixels, Point, Rectangle, Renderer, Theme, Vector, alignment,
};
use model::{dof::Dof, load::DistributedLoadDirection, node::Node};

use crate::{
    state::{
        AnalysisState, InteractionDraft, ResultDisplay, Selection, StructuralModel, WorkspaceTool,
        dof_label, element_data, element_kind,
    },
    theme,
    viewport::{DEFAULT_ZOOM, ViewportEvent, ViewportPress, ViewportState, ViewportUpdate},
};

#[derive(Debug, Clone, Copy)]
pub struct ViewportCanvas<'a> {
    model: &'a StructuralModel,
    selection: Option<Selection>,
    tool: WorkspaceTool,
    draft: InteractionDraft,
    viewport: ViewportState,
    analysis: &'a AnalysisState,
    result_display: ResultDisplay,
    result_scale: f64,
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
        analysis: &'a AnalysisState,
        result_display: ResultDisplay,
        result_scale: f64,
    ) -> Self {
        Self {
            model,
            selection,
            tool,
            draft,
            viewport,
            analysis,
            result_display,
            result_scale,
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
        self.draw_deformed_members(&mut frame, bounds);
        self.draw_members(&mut frame, bounds);
        self.draw_member_diagrams(&mut frame, bounds);
        self.draw_loads(&mut frame, bounds);
        self.draw_supports(&mut frame, bounds);
        self.draw_nodes(&mut frame, bounds);
        self.draw_result_vectors(&mut frame, bounds);
        self.draw_result_legend(&mut frame, bounds);
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

            let color = if matches!(
                self.result_display,
                ResultDisplay::MemberForces | ResultDisplay::Combined
            ) {
                self.member_force_color(id)
                    .unwrap_or(Color::from_rgb(0.812, 0.847, 0.867))
            } else if selected {
                theme::ACCENT
            } else {
                Color::from_rgb(0.812, 0.847, 0.867)
            };

            frame.stroke(
                &canvas::Path::line(a, b),
                canvas::Stroke {
                    style: canvas::Style::Solid(color),
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

    fn draw_deformed_members(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        if !matches!(
            self.result_display,
            ResultDisplay::Deformed | ResultDisplay::Displacements | ResultDisplay::Combined
        ) {
            return;
        }

        let Some(summary) = self.summary() else {
            return;
        };

        for element in &self.model.elements {
            let (_, node_i, node_j) = element_data(element);
            let Some(ni) = self.model.node(node_i) else {
                continue;
            };
            let Some(nj) = self.model.node(node_j) else {
                continue;
            };

            let a = self.deformed_node_screen_position(ni, bounds, summary);
            let b = self.deformed_node_screen_position(nj, bounds, summary);

            frame.stroke(
                &canvas::Path::line(a, b),
                canvas::Stroke {
                    style: canvas::Style::Solid(Color::from_rgb(0.992, 0.722, 0.286)),
                    width: 2.0,
                    ..canvas::Stroke::default()
                },
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
        if self.tool != WorkspaceTool::AssignLoad {
            return;
        }

        let max_distributed_load = self
            .model
            .distributed_loads
            .iter()
            .map(|load| load.magnitude.abs())
            .fold(0.0, f64::max);
        let max_point_load = self
            .model
            .nodal_loads
            .iter()
            .map(|load| load.magnitude.abs())
            .fold(0.0, f64::max);

        for load in &self.model.distributed_loads {
            let Some(element) = self.model.element(load.element_id) else {
                continue;
            };

            let (_, node_i, node_j) = element_data(element);
            let Some(ni) = self.model.node(node_i) else {
                continue;
            };
            let Some(nj) = self.model.node(node_j) else {
                continue;
            };

            let a = self.node_screen_position(ni, bounds);
            let b = self.node_screen_position(nj, bounds);
            let Some(direction) = distributed_load_direction(load.direction.clone(), a, b) else {
                continue;
            };
            if max_distributed_load <= f64::EPSILON {
                continue;
            }

            let signed = if load.magnitude >= 0.0 { 1.0 } else { -1.0 };
            let height = 18.0 + 34.0 * (load.magnitude.abs() / max_distributed_load) as f32;
            let offset = direction * signed * height;
            let qa = a + offset;
            let qb = b + offset;

            let load_area = canvas::Path::new(|builder| {
                builder.move_to(a);
                builder.line_to(qa);
                builder.line_to(qb);
                builder.line_to(b);
                builder.line_to(a);
            });
            let load_outline = canvas::Path::new(|builder| {
                builder.move_to(qa);
                builder.line_to(qb);
            });

            frame.fill(
                &load_area,
                Color::from_rgba(theme::LOAD.r, theme::LOAD.g, theme::LOAD.b, 0.13),
            );
            frame.stroke(
                &canvas::Path::line(a, b),
                canvas::Stroke {
                    style: canvas::Style::Solid(Color::from_rgba(
                        theme::LOAD.r,
                        theme::LOAD.g,
                        theme::LOAD.b,
                        0.42,
                    )),
                    width: 1.0,
                    ..canvas::Stroke::default()
                },
            );
            for (base, point) in [(a, qa), (b, qb)] {
                frame.stroke(
                    &canvas::Path::line(base, point),
                    canvas::Stroke {
                        style: canvas::Style::Solid(Color::from_rgba(
                            theme::LOAD.r,
                            theme::LOAD.g,
                            theme::LOAD.b,
                            0.62,
                        )),
                        width: 1.2,
                        ..canvas::Stroke::default()
                    },
                );
            }
            frame.stroke(
                &load_outline,
                canvas::Stroke {
                    style: canvas::Style::Solid(theme::LOAD),
                    width: 2.0,
                    ..canvas::Stroke::default()
                },
            );

            label(
                frame,
                format!("{:+.2} kN/m", load.magnitude / 1000.0),
                interpolate(qa, qb, 0.5) + direction * signed * 14.0,
                theme::LOAD,
                11.0,
                92.0,
            );
        }

        for load in &self.model.nodal_loads {
            let Some(node) = self.model.node(load.node_id) else {
                continue;
            };

            let point = self.node_screen_position(node, bounds);
            let Some(direction) = point_load_direction(load.dof) else {
                label(
                    frame,
                    format!(
                        "{} {:+.2} kN m",
                        dof_label(load.dof),
                        load.magnitude / 1000.0
                    ),
                    point + Vector::new(0.0, -32.0),
                    theme::LOAD,
                    11.0,
                    92.0,
                );
                continue;
            };

            let signed = if load.magnitude >= 0.0 { 1.0 } else { -1.0 };
            let length = if max_point_load <= f64::EPSILON {
                28.0
            } else {
                24.0 + 24.0 * (load.magnitude.abs() / max_point_load) as f32
            };
            let end = point + direction * signed * 11.0;
            let start = point + direction * signed * length;

            draw_arrow(frame, start, end, theme::LOAD, 2.0);
            label(
                frame,
                format!("{:+.2} kN", load.magnitude / 1000.0),
                start + direction * signed * 12.0,
                theme::LOAD,
                11.0,
                78.0,
            );
        }
    }

    fn draw_result_vectors(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        let Some(summary) = self.summary() else {
            return;
        };

        if matches!(
            self.result_display,
            ResultDisplay::Displacements | ResultDisplay::Combined
        ) {
            for node in &self.model.nodes {
                let start = self.node_screen_position(node, bounds);
                let delta = self.displacement_screen_vector(summary, node.id);

                if vector_length(delta) > 1.0 {
                    draw_arrow(
                        frame,
                        start,
                        start + delta,
                        Color::from_rgb(0.992, 0.722, 0.286),
                        1.8,
                    );
                }
            }
        }

        if matches!(
            self.result_display,
            ResultDisplay::Reactions | ResultDisplay::Combined
        ) {
            for reaction in &summary.reactions {
                let Some(node) = self.model.node(reaction.node_id) else {
                    continue;
                };

                let Some(direction) = reaction_direction(reaction.dof) else {
                    let point = self.node_screen_position(node, bounds);
                    label(
                        frame,
                        format!(
                            "{} {:+.2} kN",
                            dof_label(reaction.dof),
                            reaction.magnitude / 1000.0
                        ),
                        point + Vector::new(0.0, -32.0),
                        theme::SUPPORT,
                        11.0,
                        92.0,
                    );
                    continue;
                };

                let point = self.node_screen_position(node, bounds);
                let length = if summary.max_reaction <= f64::EPSILON {
                    0.0
                } else {
                    22.0 + 34.0 * (reaction.magnitude.abs() / summary.max_reaction) as f32
                };
                let signed = if reaction.magnitude >= 0.0 { 1.0 } else { -1.0 };
                let end = point + direction * signed * length;
                draw_arrow(frame, point, end, theme::SUPPORT, 2.2);
                label(
                    frame,
                    format!("{:+.2} kN", reaction.magnitude / 1000.0),
                    end + direction * signed * 12.0,
                    theme::SUPPORT,
                    11.0,
                    86.0,
                );
            }
        }
    }

    fn draw_member_diagrams(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        let Some(kind) = diagram_kind(self.result_display) else {
            return;
        };
        let Some(summary) = self.summary() else {
            return;
        };

        let max_value = match kind {
            DiagramKind::ShearY => summary.max_shear,
            DiagramKind::MomentZ => summary.max_moment,
        };
        if max_value <= f64::EPSILON {
            return;
        }

        let color = match kind {
            DiagramKind::ShearY => theme::ACCENT,
            DiagramKind::MomentZ => Color::from_rgb(0.992, 0.722, 0.286),
        };

        for diagram in summary
            .member_diagrams
            .iter()
            .filter(|diagram| diagram.kind == kind)
        {
            let Some(element) = self.model.element(diagram.element_id) else {
                continue;
            };
            let (_, node_i, node_j) = element_data(element);
            let Some(ni) = self.model.node(node_i) else {
                continue;
            };
            let Some(nj) = self.model.node(node_j) else {
                continue;
            };

            let a = self.node_screen_position(ni, bounds);
            let b = self.node_screen_position(nj, bounds);
            let axis = b - a;
            let screen_length = vector_length(axis);
            if screen_length <= f32::EPSILON || diagram.length <= f64::EPSILON {
                continue;
            }

            let unit = axis * (1.0 / screen_length);
            let normal = Vector::new(-unit.y, unit.x);
            let result_scale = self.result_screen_scale();
            let diagram_points = diagram
                .points
                .iter()
                .map(|point| {
                    let along = (point.x / diagram.length) as f32 * screen_length;
                    let offset = (point.value / max_value) as f32 * result_scale;
                    a + unit * along + normal * offset
                })
                .collect::<Vec<_>>();

            if diagram_points.len() < 2 {
                continue;
            }

            let baseline = canvas::Path::line(a, b);
            let diagram_area = canvas::Path::new(|builder| {
                builder.move_to(a);
                builder.line_to(diagram_points[0]);
                for point in diagram_points.iter().skip(1) {
                    builder.line_to(*point);
                }
                builder.line_to(b);
                builder.line_to(a);
            });
            let diagram_path = canvas::Path::new(|builder| {
                builder.move_to(diagram_points[0]);
                for point in diagram_points.iter().skip(1) {
                    builder.line_to(*point);
                }
            });

            frame.fill(
                &diagram_area,
                Color::from_rgba(color.r, color.g, color.b, 0.13),
            );

            frame.stroke(
                &baseline,
                canvas::Stroke {
                    style: canvas::Style::Solid(Color::from_rgba(color.r, color.g, color.b, 0.42)),
                    width: 1.0,
                    ..canvas::Stroke::default()
                },
            );

            for (base, point) in [(a, diagram_points[0]), (b, *diagram_points.last().unwrap())] {
                frame.stroke(
                    &canvas::Path::line(base, point),
                    canvas::Stroke {
                        style: canvas::Style::Solid(Color::from_rgba(
                            color.r, color.g, color.b, 0.62,
                        )),
                        width: 1.2,
                        ..canvas::Stroke::default()
                    },
                );
            }

            frame.stroke(
                &diagram_path,
                canvas::Stroke {
                    style: canvas::Style::Solid(color),
                    width: 2.0,
                    ..canvas::Stroke::default()
                },
            );

            if let Some((point, value)) = diagram_peak(&diagram_points, diagram) {
                label(
                    frame,
                    diagram_value_label(kind, value),
                    point + normal * 14.0,
                    color,
                    11.0,
                    88.0,
                );
            }
        }
    }

    fn draw_result_legend(&self, frame: &mut canvas::Frame, bounds: Rectangle) {
        if self.result_display == ResultDisplay::Model {
            return;
        }

        let label_text = match self.summary() {
            Some(summary) => result_legend_text(self.result_display, self.result_scale, summary),
            None => "Solve model to view results".to_string(),
        };

        label(
            frame,
            label_text,
            Point::new(bounds.width * 0.5, 22.0),
            theme::TEXT,
            12.0,
            bounds.width - 48.0,
        );
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

    fn deformed_node_screen_position(
        &self,
        node: &Node,
        bounds: Rectangle,
        summary: &crate::state::AnalysisSummary,
    ) -> Point {
        self.node_screen_position(node, bounds) + self.displacement_screen_vector(summary, node.id)
    }

    fn displacement_screen_vector(
        &self,
        summary: &crate::state::AnalysisSummary,
        node_id: usize,
    ) -> Vector {
        if summary.max_displacement <= f64::EPSILON {
            return Vector::new(0.0, 0.0);
        }

        let ux = displacement(summary, node_id, Dof::Ux);
        let uy = displacement(summary, node_id, Dof::Uy);
        let result_scale = self.result_screen_scale();

        Vector::new(
            (ux / summary.max_displacement) as f32 * result_scale,
            -(uy / summary.max_displacement) as f32 * result_scale,
        )
    }

    fn result_screen_scale(&self) -> f32 {
        self.result_scale as f32 * self.viewport.zoom / DEFAULT_ZOOM
    }

    fn member_force_color(&self, element_id: usize) -> Option<Color> {
        let summary = self.summary()?;
        if summary.max_member_force <= f64::EPSILON {
            return Some(Color::from_rgb(0.812, 0.847, 0.867));
        }

        let result = summary
            .member_results
            .iter()
            .find(|result| result.element_id == element_id)?;
        let ratio = (result.governing_force.abs() / summary.max_member_force).clamp(0.0, 1.0);

        Some(force_color(ratio))
    }

    fn summary(&self) -> Option<&crate::state::AnalysisSummary> {
        match self.analysis {
            AnalysisState::Success(summary) => Some(summary),
            AnalysisState::Idle | AnalysisState::Failed(_) => None,
        }
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

fn draw_arrow(frame: &mut canvas::Frame, start: Point, end: Point, color: Color, width: f32) {
    frame.stroke(
        &canvas::Path::line(start, end),
        canvas::Stroke {
            style: canvas::Style::Solid(color),
            width,
            ..canvas::Stroke::default()
        },
    );

    let direction = end - start;
    let length = vector_length(direction);
    if length <= f32::EPSILON {
        return;
    }

    let unit = direction * (1.0 / length);
    let normal = Vector::new(-unit.y, unit.x);
    let head = canvas::Path::new(|builder| {
        builder.move_to(end);
        builder.line_to(end - unit * 10.0 + normal * 5.5);
        builder.line_to(end - unit * 10.0 - normal * 5.5);
        builder.close();
    });

    frame.fill(&head, color);
}

fn displacement(summary: &crate::state::AnalysisSummary, node_id: usize, dof: Dof) -> f64 {
    summary
        .displacements
        .get(model::dof::global_dof_index(node_id, dof))
        .copied()
        .unwrap_or_default()
}

fn reaction_direction(dof: Dof) -> Option<Vector> {
    match dof {
        Dof::Ux => Some(Vector::new(1.0, 0.0)),
        Dof::Uy => Some(Vector::new(0.0, -1.0)),
        Dof::Uz | Dof::Rx | Dof::Ry | Dof::Rz => None,
    }
}

fn point_load_direction(dof: Dof) -> Option<Vector> {
    match dof {
        Dof::Ux => Some(Vector::new(1.0, 0.0)),
        Dof::Uy => Some(Vector::new(0.0, -1.0)),
        Dof::Uz | Dof::Rx | Dof::Ry | Dof::Rz => None,
    }
}

fn diagram_kind(display: ResultDisplay) -> Option<DiagramKind> {
    match display {
        ResultDisplay::ShearForce => Some(DiagramKind::ShearY),
        ResultDisplay::BendingMoment => Some(DiagramKind::MomentZ),
        ResultDisplay::Model
        | ResultDisplay::Deformed
        | ResultDisplay::Displacements
        | ResultDisplay::Reactions
        | ResultDisplay::MemberForces
        | ResultDisplay::Combined => None,
    }
}

fn diagram_peak<'a>(
    screen_points: &'a [Point],
    diagram: &bajrang_core::post::diagrams::MemberDiagram,
) -> Option<(Point, f64)> {
    let (index, point) = diagram
        .points
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.value.abs().total_cmp(&b.value.abs()))?;

    Some((*screen_points.get(index)?, point.value))
}

fn diagram_value_label(kind: DiagramKind, value: f64) -> String {
    match kind {
        DiagramKind::ShearY => format!("{:+.2} kN", value / 1000.0),
        DiagramKind::MomentZ => format!("{:+.2} kN m", value / 1000.0),
    }
}

fn result_legend_text(
    display: ResultDisplay,
    scale: f64,
    summary: &crate::state::AnalysisSummary,
) -> String {
    match display {
        ResultDisplay::ShearForce => {
            format!(
                "Shear  |  scale {scale:.0}  |  max |V| {:.2} kN",
                summary.max_shear / 1000.0
            )
        }
        ResultDisplay::BendingMoment => {
            format!(
                "Moment  |  scale {scale:.0}  |  max |M| {:.2} kN m",
                summary.max_moment / 1000.0
            )
        }
        ResultDisplay::Model
        | ResultDisplay::Deformed
        | ResultDisplay::Displacements
        | ResultDisplay::Reactions
        | ResultDisplay::MemberForces
        | ResultDisplay::Combined => format!(
            "{}  |  scale {scale:.0}  |  max |u| {:.2e} m",
            display.label(),
            summary.max_displacement
        ),
    }
}

fn force_color(ratio: f64) -> Color {
    let t = ratio as f32;
    Color::from_rgb(0.290 + 0.667 * t, 0.761 - 0.388 * t, 0.612 - 0.290 * t)
}

fn vector_length(vector: Vector) -> f32 {
    (vector.x * vector.x + vector.y * vector.y).sqrt()
}

fn interpolate(a: Point, b: Point, t: f32) -> Point {
    Point::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

fn unit_vector(vector: Vector) -> Option<Vector> {
    let length = vector_length(vector);
    (length > f32::EPSILON).then_some(vector * (1.0 / length))
}

fn perpendicular_unit(a: Point, b: Point) -> Option<Vector> {
    let unit = unit_vector(b - a)?;
    Some(Vector::new(-unit.y, unit.x))
}

fn distributed_load_direction(
    direction: DistributedLoadDirection,
    a: Point,
    b: Point,
) -> Option<Vector> {
    match direction {
        DistributedLoadDirection::LocalX => unit_vector(b - a),
        DistributedLoadDirection::LocalY => perpendicular_unit(a, b),
        DistributedLoadDirection::GlobalX => Some(Vector::new(1.0, 0.0)),
        DistributedLoadDirection::GlobalY => Some(Vector::new(0.0, -1.0)),
        DistributedLoadDirection::LocalZ | DistributedLoadDirection::GlobalZ => None,
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
