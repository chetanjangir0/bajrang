#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoordinateAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemberEndpoint {
    Start,
    End,
}

impl MemberEndpoint {
    pub fn label(self) -> &'static str {
        match self {
            Self::Start => "i",
            Self::End => "j",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadTarget {
    Node(usize),
    Element(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadKind {
    Point,
    Distributed,
}

impl LoadKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Point => "Point",
            Self::Distributed => "Distributed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadBuilder {
    pub target: LoadTarget,
    pub kind: LoadKind,
    pub dof: model::dof::Dof,
    pub direction: model::load::DistributedLoadDirection,
    pub magnitude: String,
}

impl LoadBuilder {
    pub fn point(node_id: usize) -> Self {
        Self {
            target: LoadTarget::Node(node_id),
            kind: LoadKind::Point,
            dof: model::dof::Dof::Uy,
            direction: model::load::DistributedLoadDirection::GlobalY,
            magnitude: "-10".to_string(),
        }
    }

    pub fn distributed(element_id: usize) -> Self {
        Self {
            target: LoadTarget::Element(element_id),
            kind: LoadKind::Distributed,
            dof: model::dof::Dof::Uy,
            direction: model::load::DistributedLoadDirection::GlobalY,
            magnitude: "-5".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportPreset {
    Pin,
    Fixed,
    Roller,
}

impl SupportPreset {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pin => "Pin",
            Self::Fixed => "Fixed",
            Self::Roller => "Roller",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SupportBuilder {
    pub node_id: usize,
    pub ux: bool,
    pub uy: bool,
    pub uz: bool,
    pub rx: bool,
    pub ry: bool,
    pub rz: bool,
}

impl SupportBuilder {
    pub fn new(node_id: usize) -> Self {
        Self {
            node_id,
            ux: true,
            uy: true,
            uz: true,
            rx: false,
            ry: false,
            rz: false,
        }
    }
}

impl CoordinateAxis {
    pub fn label(self) -> &'static str {
        match self {
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
        }
    }
}
