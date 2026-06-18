#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceTool {
    Select,
    AddNode,
    DrawMember,
    AssignLoad,
    AssignSupport,
    Analyze,
}

impl WorkspaceTool {
    pub const ALL: [Self; 6] = [
        Self::Select,
        Self::AddNode,
        Self::DrawMember,
        Self::AssignLoad,
        Self::AssignSupport,
        Self::Analyze,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::AddNode => "Node",
            Self::DrawMember => "Member",
            Self::AssignLoad => "Load",
            Self::AssignSupport => "Support",
            Self::Analyze => "Analyze",
        }
    }

    pub fn marker(self) -> &'static str {
        match self {
            Self::Select => "S",
            Self::AddNode => "+",
            Self::DrawMember => "M",
            Self::AssignLoad => "F",
            Self::AssignSupport => "P",
            Self::Analyze => "A",
        }
    }
}
