use bajrang_core::analysis::linear_static;
use model::{
    boundary::Support, dof::Dof, elements::frame2d::Frame2D, load::NodalLoad, material::Material,
    node::Node, section::Section,
};

fn displacement(results: &linear_static::Frame2DResults, node: usize, dof: Dof) -> f64 {
    results.displacements[node * 3 + dof as usize]
}

fn assert_close(actual: f64, expected: f64, tol: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= tol,
        "{label}: expected {expected:.12e}, got {actual:.12e}"
    );
}

#[test]
fn vertical_cantilever_horizontal_tip_load_uses_global_transformation() {
    let nodes = vec![Node::new(0, 0.0, 0.0), Node::new(1, 0.0, 3.0)];
    let material = Material::new(210.0e9, 0.3);
    let section = Section::new(0.02, 1.6e-5);
    let elements = vec![Frame2D::new(0, 0, 1, material, section)];

    let supports = vec![
        Support::new(0, Dof::Ux),
        Support::new(0, Dof::Uy),
        Support::new(0, Dof::Rz),
    ];
    let loads = vec![NodalLoad::new(1, Dof::Ux, 5_000.0)];

    let results = linear_static::run_frame2d(&nodes, &elements, &supports, &loads)
        .expect("Frame analysis should succeed");

    assert_close(
        displacement(&results, 1, Dof::Ux),
        0.013392857142857143,
        1e-12,
        "Tip horizontal displacement",
    );
    assert_close(displacement(&results, 1, Dof::Uy), 0.0, 1e-12, "Tip vertical displacement");
    assert_close(
        displacement(&results, 1, Dof::Rz),
        -0.006696428571428572,
        1e-12,
        "Tip rotation",
    );

    let end_forces = results.member_end_forces[0];
    assert_close(end_forces[0], 0.0, 1e-6, "Node i axial force");
    assert_close(end_forces[1], 5_000.0, 1e-6, "Node i shear");
    assert_close(end_forces[2], 15_000.0, 1e-5, "Node i moment");
    assert_close(end_forces[3], 0.0, 1e-6, "Node j axial force");
    assert_close(end_forces[4], -5_000.0, 1e-6, "Node j shear");
    assert_close(end_forces[5], 0.0, 1e-5, "Node j moment");
}
