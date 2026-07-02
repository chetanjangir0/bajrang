use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParametricSectionKind {
    Rectangle,
    Circle,
    HollowRectangle,
    HollowCircle,
    ISection,
}

impl ParametricSectionKind {
    pub const ALL: [Self; 5] = [
        Self::Rectangle,
        Self::Circle,
        Self::HollowRectangle,
        Self::HollowCircle,
        Self::ISection,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Rectangle => "Rectangle",
            Self::Circle => "Circle",
            Self::HollowRectangle => "Hollow rectangle",
            Self::HollowCircle => "Hollow circle",
            Self::ISection => "I-section",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParametricSection {
    Rectangle {
        width: f64,
        depth: f64,
    },
    Circle {
        diameter: f64,
    },
    HollowRectangle {
        width: f64,
        depth: f64,
        wall_thickness: f64,
    },
    HollowCircle {
        outer_diameter: f64,
        wall_thickness: f64,
    },
    ISection {
        depth: f64,
        flange_width: f64,
        web_thickness: f64,
        flange_thickness: f64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SectionError {
    #[error("{field} must be positive")]
    NonPositive { field: &'static str },
    #[error("{field} is too large for {section}")]
    InvalidThickness {
        section: &'static str,
        field: &'static str,
    },
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

    pub fn from_parametric(parametric: ParametricSection) -> Result<Self, SectionError> {
        parametric.properties()
    }
}

impl ParametricSection {
    pub fn kind(&self) -> ParametricSectionKind {
        match self {
            Self::Rectangle { .. } => ParametricSectionKind::Rectangle,
            Self::Circle { .. } => ParametricSectionKind::Circle,
            Self::HollowRectangle { .. } => ParametricSectionKind::HollowRectangle,
            Self::HollowCircle { .. } => ParametricSectionKind::HollowCircle,
            Self::ISection { .. } => ParametricSectionKind::ISection,
        }
    }

    pub fn properties(&self) -> Result<Section, SectionError> {
        match *self {
            Self::Rectangle { width, depth } => rectangle(width, depth),
            Self::Circle { diameter } => circle(diameter),
            Self::HollowRectangle {
                width,
                depth,
                wall_thickness,
            } => hollow_rectangle(width, depth, wall_thickness),
            Self::HollowCircle {
                outer_diameter,
                wall_thickness,
            } => hollow_circle(outer_diameter, wall_thickness),
            Self::ISection {
                depth,
                flange_width,
                web_thickness,
                flange_thickness,
            } => i_section(depth, flange_width, web_thickness, flange_thickness),
        }
    }
}

fn rectangle(width: f64, depth: f64) -> Result<Section, SectionError> {
    positive("width", width)?;
    positive("depth", depth)?;

    let area = width * depth;
    let iy = depth * width.powi(3) / 12.0;
    let iz = width * depth.powi(3) / 12.0;
    let torsional_constant = rectangle_torsion_constant(width, depth);

    Ok(Section::new_3d(area, iy, iz, torsional_constant))
}

fn circle(diameter: f64) -> Result<Section, SectionError> {
    positive("diameter", diameter)?;

    let area = PI * diameter.powi(2) / 4.0;
    let i = PI * diameter.powi(4) / 64.0;
    let torsional_constant = PI * diameter.powi(4) / 32.0;

    Ok(Section::new_3d(area, i, i, torsional_constant))
}

fn hollow_rectangle(width: f64, depth: f64, wall_thickness: f64) -> Result<Section, SectionError> {
    positive("width", width)?;
    positive("depth", depth)?;
    positive("wall thickness", wall_thickness)?;

    if 2.0 * wall_thickness >= width {
        return Err(SectionError::InvalidThickness {
            section: "hollow rectangle",
            field: "wall thickness",
        });
    }

    if 2.0 * wall_thickness >= depth {
        return Err(SectionError::InvalidThickness {
            section: "hollow rectangle",
            field: "wall thickness",
        });
    }

    let inner_width = width - 2.0 * wall_thickness;
    let inner_depth = depth - 2.0 * wall_thickness;
    let area = width * depth - inner_width * inner_depth;
    let iy = (depth * width.powi(3) - inner_depth * inner_width.powi(3)) / 12.0;
    let iz = (width * depth.powi(3) - inner_width * inner_depth.powi(3)) / 12.0;
    let torsional_constant = 2.0
        * wall_thickness
        * wall_thickness
        * (width - wall_thickness).powi(2)
        * (depth - wall_thickness).powi(2)
        / (width + depth - 2.0 * wall_thickness);

    Ok(Section::new_3d(area, iy, iz, torsional_constant))
}

fn hollow_circle(outer_diameter: f64, wall_thickness: f64) -> Result<Section, SectionError> {
    positive("outer diameter", outer_diameter)?;
    positive("wall thickness", wall_thickness)?;

    if 2.0 * wall_thickness >= outer_diameter {
        return Err(SectionError::InvalidThickness {
            section: "hollow circle",
            field: "wall thickness",
        });
    }

    let inner_diameter = outer_diameter - 2.0 * wall_thickness;
    let area = PI * (outer_diameter.powi(2) - inner_diameter.powi(2)) / 4.0;
    let i = PI * (outer_diameter.powi(4) - inner_diameter.powi(4)) / 64.0;
    let torsional_constant = PI * (outer_diameter.powi(4) - inner_diameter.powi(4)) / 32.0;

    Ok(Section::new_3d(area, i, i, torsional_constant))
}

fn i_section(
    depth: f64,
    flange_width: f64,
    web_thickness: f64,
    flange_thickness: f64,
) -> Result<Section, SectionError> {
    positive("depth", depth)?;
    positive("flange width", flange_width)?;
    positive("web thickness", web_thickness)?;
    positive("flange thickness", flange_thickness)?;

    if web_thickness >= flange_width {
        return Err(SectionError::InvalidThickness {
            section: "I-section",
            field: "web thickness",
        });
    }

    if 2.0 * flange_thickness >= depth {
        return Err(SectionError::InvalidThickness {
            section: "I-section",
            field: "flange thickness",
        });
    }

    let web_depth = depth - 2.0 * flange_thickness;
    let area = 2.0 * flange_width * flange_thickness + web_thickness * web_depth;
    let iy = 2.0 * flange_thickness * flange_width.powi(3) / 12.0
        + web_depth * web_thickness.powi(3) / 12.0;
    let iz =
        (flange_width * depth.powi(3) - (flange_width - web_thickness) * web_depth.powi(3)) / 12.0;
    let torsional_constant =
        (2.0 * flange_width * flange_thickness.powi(3) + web_depth * web_thickness.powi(3)) / 3.0;

    Ok(Section::new_3d(area, iy, iz, torsional_constant))
}

fn rectangle_torsion_constant(width: f64, depth: f64) -> f64 {
    let (longer, shorter) = if width >= depth {
        (width, depth)
    } else {
        (depth, width)
    };
    let ratio = shorter / longer;

    longer * shorter.powi(3) * (1.0 / 3.0 - 0.21 * ratio * (1.0 - ratio.powi(4) / 12.0))
}

fn positive(field: &'static str, value: f64) -> Result<(), SectionError> {
    if value > 0.0 {
        Ok(())
    } else {
        Err(SectionError::NonPositive { field })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-12;

    #[test]
    fn rectangle_properties_are_computed_from_geometry() {
        let section = ParametricSection::Rectangle {
            width: 0.2,
            depth: 0.4,
        }
        .properties()
        .expect("rectangle section should be valid");

        assert_close(section.area, 0.08);
        assert_close(section.moment_of_inertia_y, 0.4 * 0.2_f64.powi(3) / 12.0);
        assert_close(section.moment_of_inertia_z, 0.2 * 0.4_f64.powi(3) / 12.0);
        assert_eq!(section.moment_of_inertia, section.moment_of_inertia_z);
    }

    #[test]
    fn circular_torsion_constant_is_polar_second_moment() {
        let section = ParametricSection::Circle { diameter: 0.3 }
            .properties()
            .expect("circle section should be valid");

        assert_close(section.area, PI * 0.3_f64.powi(2) / 4.0);
        assert_close(section.moment_of_inertia_y, PI * 0.3_f64.powi(4) / 64.0);
        assert_close(section.torsional_constant, PI * 0.3_f64.powi(4) / 32.0);
    }

    #[test]
    fn hollow_rectangle_rejects_impossible_wall_thickness() {
        let error = ParametricSection::HollowRectangle {
            width: 0.2,
            depth: 0.4,
            wall_thickness: 0.1,
        }
        .properties()
        .expect_err("wall thickness should be invalid");

        assert_eq!(
            error,
            SectionError::InvalidThickness {
                section: "hollow rectangle",
                field: "wall thickness"
            }
        );
    }

    #[test]
    fn i_section_uses_flange_and_web_subtraction() {
        let section = ParametricSection::ISection {
            depth: 0.3,
            flange_width: 0.15,
            web_thickness: 0.01,
            flange_thickness: 0.02,
        }
        .properties()
        .expect("I-section should be valid");

        assert_close(section.area, 0.0086);
        assert!(section.moment_of_inertia_z > section.moment_of_inertia_y);
        assert!(section.torsional_constant > 0.0);
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() <= EPS,
            "actual {actual}, expected {expected}"
        );
    }
}
