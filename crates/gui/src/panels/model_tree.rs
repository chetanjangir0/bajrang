use iced::widget::{column, container, text};
use iced::{Element, Fill};
use model::{dof::Dof, elements::StructuralElement};

use crate::{
    app::Message,
    state::{Selection, StructuralModel},
    theme,
};

pub fn view(model: &StructuralModel, selection: Option<Selection>) -> Element<'_, Message> {
    let nodes = model.nodes.iter().fold(
        column![section_title("Nodes")].spacing(4),
        |column, node| {
            column.push(row_item(
                format!("N{}  ({:.2}, {:.2})", node.id, node.x, node.y),
                selection == Some(Selection::Node(node.id)),
            ))
        },
    );

    let elements = model.elements.iter().fold(
        column![section_title("Elements")].spacing(4),
        |column, element| {
            column.push(row_item(
                element_label(element),
                selection == Some(Selection::Element(element_id(element))),
            ))
        },
    );

    let supports = model.supports.iter().fold(
        column![section_title("Supports")].spacing(4),
        |column, support| {
            column.push(text(format!("N{} {}", support.node_id, dof_label(support.dof))).size(14))
        },
    );

    let loads = model.nodal_loads.iter().fold(
        column![section_title("Loads")].spacing(4),
        |column, load| {
            column.push(
                text(format!(
                    "N{} {} {:+.2e}",
                    load.node_id,
                    dof_label(load.dof),
                    load.magnitude
                ))
                .size(14),
            )
        },
    );

    column![panel_header("Model"), nodes, elements, supports, loads,]
        .spacing(18)
        .padding(14)
        .width(Fill)
        .into()
}

fn panel_header(label: &str) -> Element<'_, Message> {
    text(label).size(18).color(theme::TEXT).into()
}

fn section_title(label: &str) -> Element<'_, Message> {
    text(label).size(13).color(theme::TEXT_MUTED).into()
}

fn row_item(label: String, selected: bool) -> Element<'static, Message> {
    container(text(label).size(14).color(theme::TEXT))
        .padding([6, 8])
        .width(Fill)
        .style(if selected {
            theme::selected_row
        } else {
            theme::neutral_row
        })
        .into()
}

fn element_id(element: &StructuralElement) -> usize {
    match element {
        StructuralElement::Truss2D(element) => element.id,
        StructuralElement::Truss3D(element) => element.id,
        StructuralElement::Beam2D(element) => element.id,
        StructuralElement::Beam3D(element) => element.id,
        StructuralElement::Frame2D(element) => element.id,
        StructuralElement::Frame3D(element) => element.id,
    }
}

fn element_label(element: &StructuralElement) -> String {
    match element {
        StructuralElement::Truss2D(element) => {
            format!("T{}  N{} - N{}", element.id, element.node_i, element.node_j)
        }
        StructuralElement::Truss3D(element) => {
            format!(
                "T3{}  N{} - N{}",
                element.id, element.node_i, element.node_j
            )
        }
        StructuralElement::Beam2D(element) => {
            format!("B{}  N{} - N{}", element.id, element.node_i, element.node_j)
        }
        StructuralElement::Beam3D(element) => {
            format!(
                "B3{}  N{} - N{}",
                element.id, element.node_i, element.node_j
            )
        }
        StructuralElement::Frame2D(element) => {
            format!("F{}  N{} - N{}", element.id, element.node_i, element.node_j)
        }
        StructuralElement::Frame3D(element) => {
            format!(
                "F3{}  N{} - N{}",
                element.id, element.node_i, element.node_j
            )
        }
    }
}

fn dof_label(dof: Dof) -> &'static str {
    match dof {
        Dof::Ux => "Ux",
        Dof::Uy => "Uy",
        Dof::Uz => "Uz",
        Dof::Rx => "Rx",
        Dof::Ry => "Ry",
        Dof::Rz => "Rz",
    }
}
