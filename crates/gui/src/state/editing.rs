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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoadField {
    Node,
    Dof,
    Magnitude,
}

impl LoadField {
    pub fn label(self) -> &'static str {
        match self {
            Self::Node => "N",
            Self::Dof => "DOF",
            Self::Magnitude => "kN",
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
