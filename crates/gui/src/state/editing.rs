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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SupportField {
    Node,
    Dof,
}

impl SupportField {
    pub fn label(self) -> &'static str {
        match self {
            Self::Node => "N",
            Self::Dof => "DOF",
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
