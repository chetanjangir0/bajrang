#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    Node(usize),
    Element(usize),
}

impl Selection {
    pub fn label(self) -> String {
        match self {
            Self::Node(id) => format!("Node {id}"),
            Self::Element(id) => format!("Member {id}"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct InteractionDraft {
    pub member_start: Option<usize>,
}

impl InteractionDraft {
    pub fn clear(&mut self) {
        self.member_start = None;
    }
}
