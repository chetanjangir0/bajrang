use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub area: f64,
    pub moment_of_inertia: f64,
}

impl Section {
    pub fn new(area: f64, moment_of_inertia: f64) -> Self {
        assert!(area > 0.0, "Area must be positive");
        Self {
            area,
            moment_of_inertia,
        }
    }

    pub fn truss(area: f64) -> Self {
        Self::new(area, 0.0)
    }
}
