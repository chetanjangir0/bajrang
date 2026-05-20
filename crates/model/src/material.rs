use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
    pub elastic_modulus: f64,
    pub poisson_ratio: f64,
}

impl Material {
    pub fn new(elastic_modulus: f64, poisson_ratio: f64) -> Self {
        assert!(elastic_modulus > 0.0, "E must be positive");
        assert!(
            (0.0..0.5).contains(&poisson_ratio),
            "Poisson's ratio must be in [0, 0.5)"
        );
        Self {
            elastic_modulus,
            poisson_ratio,
        }
    }

    pub fn steel() -> Self {
        Self::new(200e9, 0.3)
    }
}
