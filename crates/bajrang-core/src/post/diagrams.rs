use model::{
    elements::{StructuralElement, beam2d::Beam2D, frame2d::Frame2D},
    load::{DistributedLoad, DistributedLoadDirection},
    node::Node,
};

use crate::analysis::linear_static::ElementResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagramKind {
    ShearY,
    MomentZ,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiagramPoint {
    pub x: f64,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemberDiagram {
    pub element_id: usize,
    pub kind: DiagramKind,
    pub length: f64,
    pub points: Vec<DiagramPoint>,
}

impl MemberDiagram {
    pub fn max_abs_value(&self) -> f64 {
        self.points
            .iter()
            .fold(0.0, |max, point| f64::max(max, point.value.abs()))
    }
}

pub fn beam2d_diagrams(
    nodes: &[Node],
    elements: &[Beam2D],
    end_forces: &[[f64; 4]],
    distributed_loads: &[DistributedLoad],
    stations: usize,
) -> Vec<MemberDiagram> {
    elements
        .iter()
        .zip(end_forces.iter())
        .flat_map(|(element, end_forces)| {
            let length = element
                .geometry(&nodes[element.node_i], &nodes[element.node_j])
                .length;
            let wy = beam2d_transverse_load(element.id, distributed_loads);
            let fixed = beam2d_equivalent_transverse_loads(length, wy);
            let shear_i = end_forces[0] - fixed[0];
            let moment_i = end_forces[1] - fixed[1];

            member_diagrams(element.id, length, shear_i, moment_i, wy, stations)
        })
        .collect()
}

pub fn frame2d_diagrams(
    nodes: &[Node],
    elements: &[Frame2D],
    end_forces: &[[f64; 6]],
    distributed_loads: &[DistributedLoad],
    stations: usize,
) -> Vec<MemberDiagram> {
    elements
        .iter()
        .zip(end_forces.iter())
        .flat_map(|(element, end_forces)| {
            let geom = element.geometry(&nodes[element.node_i], &nodes[element.node_j]);
            let wy = frame2d_transverse_load(element.id, geom.cos, geom.sin, distributed_loads);
            let fixed = frame2d_equivalent_transverse_loads(geom.length, wy);
            let shear_i = end_forces[1] - fixed[1];
            let moment_i = end_forces[2] - fixed[2];

            member_diagrams(element.id, geom.length, shear_i, moment_i, wy, stations)
        })
        .collect()
}

pub fn mixed_2d_diagrams(
    nodes: &[Node],
    elements: &[StructuralElement],
    results: &[ElementResult],
    distributed_loads: &[DistributedLoad],
    stations: usize,
) -> Vec<MemberDiagram> {
    elements
        .iter()
        .zip(results.iter())
        .flat_map(|(element, result)| match (element, result) {
            (StructuralElement::Beam2D(element), ElementResult::Beam2D { end_forces }) => {
                let length = element
                    .geometry(&nodes[element.node_i], &nodes[element.node_j])
                    .length;
                let wy = beam2d_transverse_load(element.id, distributed_loads);
                let fixed = beam2d_equivalent_transverse_loads(length, wy);
                let shear_i = end_forces[0] - fixed[0];
                let moment_i = end_forces[1] - fixed[1];

                member_diagrams(element.id, length, shear_i, moment_i, wy, stations)
            }
            (StructuralElement::Frame2D(element), ElementResult::Frame2D { end_forces }) => {
                let geom = element.geometry(&nodes[element.node_i], &nodes[element.node_j]);
                let wy = frame2d_transverse_load(element.id, geom.cos, geom.sin, distributed_loads);
                let fixed = frame2d_equivalent_transverse_loads(geom.length, wy);
                let shear_i = end_forces[1] - fixed[1];
                let moment_i = end_forces[2] - fixed[2];

                member_diagrams(element.id, geom.length, shear_i, moment_i, wy, stations)
            }
            _ => Vec::new(),
        })
        .collect()
}

fn member_diagrams(
    element_id: usize,
    length: f64,
    shear_i: f64,
    moment_i: f64,
    transverse_load: f64,
    stations: usize,
) -> Vec<MemberDiagram> {
    let stations = stations.max(2);
    let mut shear = Vec::with_capacity(stations);
    let mut moment = Vec::with_capacity(stations);

    for station in 0..stations {
        let ratio = station as f64 / (stations - 1) as f64;
        let x = ratio * length;

        // Positive internal moment follows the existing element end-force convention.
        shear.push(DiagramPoint {
            x,
            value: shear_i + transverse_load * x,
        });
        moment.push(DiagramPoint {
            x,
            value: moment_i - shear_i * x - 0.5 * transverse_load * x * x,
        });
    }

    vec![
        MemberDiagram {
            element_id,
            kind: DiagramKind::ShearY,
            length,
            points: shear,
        },
        MemberDiagram {
            element_id,
            kind: DiagramKind::MomentZ,
            length,
            points: moment,
        },
    ]
}

fn beam2d_transverse_load(element_id: usize, distributed_loads: &[DistributedLoad]) -> f64 {
    distributed_loads
        .iter()
        .filter(|load| load.element_id == element_id)
        .map(|load| match load.direction {
            DistributedLoadDirection::LocalY | DistributedLoadDirection::GlobalY => load.magnitude,
            DistributedLoadDirection::LocalX
            | DistributedLoadDirection::GlobalX
            | DistributedLoadDirection::LocalZ
            | DistributedLoadDirection::GlobalZ => 0.0,
        })
        .sum()
}

fn frame2d_transverse_load(
    element_id: usize,
    cos: f64,
    sin: f64,
    distributed_loads: &[DistributedLoad],
) -> f64 {
    distributed_loads
        .iter()
        .filter(|load| load.element_id == element_id)
        .map(|load| match load.direction {
            DistributedLoadDirection::LocalY => load.magnitude,
            DistributedLoadDirection::GlobalX => -sin * load.magnitude,
            DistributedLoadDirection::GlobalY => cos * load.magnitude,
            DistributedLoadDirection::LocalX
            | DistributedLoadDirection::LocalZ
            | DistributedLoadDirection::GlobalZ => 0.0,
        })
        .sum()
}

fn beam2d_equivalent_transverse_loads(length: f64, wy: f64) -> [f64; 4] {
    [
        wy * length / 2.0,
        wy * length * length / 12.0,
        wy * length / 2.0,
        -wy * length * length / 12.0,
    ]
}

fn frame2d_equivalent_transverse_loads(length: f64, wy: f64) -> [f64; 6] {
    [
        0.0,
        wy * length / 2.0,
        wy * length * length / 12.0,
        0.0,
        wy * length / 2.0,
        -wy * length * length / 12.0,
    ]
}
