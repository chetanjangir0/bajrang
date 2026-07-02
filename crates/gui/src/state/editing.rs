use model::section::ParametricSectionKind;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionField {
    Width,
    Depth,
    Diameter,
    WallThickness,
    OuterDiameter,
    FlangeWidth,
    WebThickness,
    FlangeThickness,
}

impl SectionField {
    pub fn label(self) -> &'static str {
        match self {
            Self::Width => "Width",
            Self::Depth => "Depth",
            Self::Diameter => "Diameter",
            Self::WallThickness => "Wall",
            Self::OuterDiameter => "Outer dia.",
            Self::FlangeWidth => "Flange width",
            Self::WebThickness => "Web",
            Self::FlangeThickness => "Flange",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SectionBuilder {
    pub element_id: usize,
    pub kind: ParametricSectionKind,
    pub width: String,
    pub depth: String,
    pub diameter: String,
    pub wall_thickness: String,
    pub outer_diameter: String,
    pub flange_width: String,
    pub web_thickness: String,
    pub flange_thickness: String,
}

impl SectionBuilder {
    pub fn new(element_id: usize) -> Self {
        Self {
            element_id,
            kind: ParametricSectionKind::Rectangle,
            width: "0.20".to_string(),
            depth: "0.30".to_string(),
            diameter: "0.25".to_string(),
            wall_thickness: "0.01".to_string(),
            outer_diameter: "0.25".to_string(),
            flange_width: "0.15".to_string(),
            web_thickness: "0.01".to_string(),
            flange_thickness: "0.02".to_string(),
        }
    }

    pub fn field_value(&self, field: SectionField) -> &str {
        match field {
            SectionField::Width => &self.width,
            SectionField::Depth => &self.depth,
            SectionField::Diameter => &self.diameter,
            SectionField::WallThickness => &self.wall_thickness,
            SectionField::OuterDiameter => &self.outer_diameter,
            SectionField::FlangeWidth => &self.flange_width,
            SectionField::WebThickness => &self.web_thickness,
            SectionField::FlangeThickness => &self.flange_thickness,
        }
    }

    pub fn set_field_value(&mut self, field: SectionField, value: String) {
        match field {
            SectionField::Width => self.width = value,
            SectionField::Depth => self.depth = value,
            SectionField::Diameter => self.diameter = value,
            SectionField::WallThickness => self.wall_thickness = value,
            SectionField::OuterDiameter => self.outer_diameter = value,
            SectionField::FlangeWidth => self.flange_width = value,
            SectionField::WebThickness => self.web_thickness = value,
            SectionField::FlangeThickness => self.flange_thickness = value,
        }
    }
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
