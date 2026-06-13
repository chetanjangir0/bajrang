use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub area: f64,
    pub moment_of_inertia: f64,
    #[serde(default)]
    pub moment_of_inertia_y: f64,
    #[serde(default)]
    pub moment_of_inertia_z: f64,
    #[serde(default)]
    pub torsional_constant: f64,
}

impl Section {
    pub fn new(area: f64, moment_of_inertia: f64) -> Self {
        assert!(area > 0.0, "Area must be positive");
        Self {
            area,
            moment_of_inertia,
            moment_of_inertia_y: moment_of_inertia,
            moment_of_inertia_z: moment_of_inertia,
            torsional_constant: 2.0 * moment_of_inertia,
        }
    }

    pub fn new_3d(
        area: f64,
        moment_of_inertia_y: f64,
        moment_of_inertia_z: f64,
        torsional_constant: f64,
    ) -> Self {
        assert!(area > 0.0, "Area must be positive");
        assert!(
            moment_of_inertia_y > 0.0,
            "3D section requires a positive local-y second moment of area"
        );
        assert!(
            moment_of_inertia_z > 0.0,
            "3D section requires a positive local-z second moment of area"
        );
        assert!(
            torsional_constant > 0.0,
            "3D section requires a positive torsional constant"
        );

        Self {
            area,
            moment_of_inertia: moment_of_inertia_z,
            moment_of_inertia_y,
            moment_of_inertia_z,
            torsional_constant,
        }
    }

    pub fn truss(area: f64) -> Self {
        Self::new(area, 0.0)
    }
}
