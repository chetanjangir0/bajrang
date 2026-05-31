use bajrang_core::analysis::linear_static::{self, ElementResult};
use model::{
    boundary::Support,
    dof::Dof,
    elements::{beam2d::Beam2D, truss2d::Truss2D, StructuralElement},
    load::NodalLoad,
    material::Material,
    node::Node,
    section::Section,
};

fn displacement(results: &linear_static::Mixed2DResults, node: usize, dof: Dof) -> f64 {
    results.displacements[node * 3 + dof as usize]
}

fn assert_close(actual: f64, expected: f64, tol: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= tol,
        "{label}: expected {expected:.12e}, got {actual:.12e}"
    );
}

#[test]
fn mixed_beam_and_truss_share_one_solution() {
    let nodes = vec![Node::new(0, 0.0, 0.0), Node::new(1, 2.0, 0.0)];

    let truss = Truss2D::new(
        0,
        0,
        1,
        Material::new(200.0e9, 0.3),
        Section::truss(0.01),
    );
    let beam = Beam2D::new(
        1,
        0,
        1,
        Material::new(200.0e9, 0.3),
        Section::new(0.02, 8.0e-6),
    );

    let elements = vec![
        StructuralElement::Truss2D(truss),
        StructuralElement::Beam2D(beam),
    ];

    let supports = vec![
        Support::new(0, Dof::Ux),
        Support::new(0, Dof::Uy),
        Support::new(0, Dof::Rz),
    ];
    let nodal_loads = vec![
        NodalLoad::new(1, Dof::Ux, 10_000.0),
        NodalLoad::new(1, Dof::Uy, -1_000.0),
    ];

    let results = linear_static::run_mixed(&nodes, &elements, &supports, &nodal_loads, &[])
        .expect("Mixed analysis should succeed");

    assert_close(displacement(&results, 1, Dof::Ux), 1.0e-5, 1e-12, "Node 1 Ux");
    assert_close(displacement(&results, 1, Dof::Uy), -1.0 / 600.0, 1e-12, "Node 1 Uy");
    assert_close(displacement(&results, 1, Dof::Rz), -0.00125, 1e-12, "Node 1 Rz");

    match &results.member_results[0] {
        ElementResult::Truss2D { axial_force } => {
            assert_close(*axial_force, 10_000.0, 1e-6, "Truss axial force");
        }
        _ => panic!("Expected truss result for first member"),
    }

    match &results.member_results[1] {
        ElementResult::Beam2D { end_forces } => {
            assert_close(end_forces[0], 1_000.0, 1e-6, "Beam node i shear");
            assert_close(end_forces[1], 2_000.0, 1e-6, "Beam node i moment");
            assert_close(end_forces[2], -1_000.0, 1e-6, "Beam node j shear");
            assert_close(end_forces[3], 0.0, 1e-6, "Beam node j moment");
        }
        _ => panic!("Expected beam result for second member"),
    }
}
