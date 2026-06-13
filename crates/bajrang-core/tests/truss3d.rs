use bajrang_core::analysis::linear_static;
use model::{
    boundary::Support,
    dof::{Dof, global_dof_index},
    elements::truss3d::Truss3D,
    load::NodalLoad,
    material::Material,
    node::Node,
    section::Section,
};

fn displacement(results: &linear_static::Truss3DResults, node: usize, dof: Dof) -> f64 {
    results.displacements[global_dof_index(node, dof)]
}

fn reaction(results: &linear_static::Truss3DResults, node: usize, dof: Dof) -> f64 {
    results
        .support_reactions
        .iter()
        .find(|reaction| reaction.node_id == node && reaction.dof == dof)
        .map(|reaction| reaction.magnitude)
        .expect("expected support reaction")
}

fn assert_close(actual: f64, expected: f64, tol: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= tol,
        "{label}: expected {expected:.12e}, got {actual:.12e}"
    );
}

#[test]
fn orthogonal_three_bar_space_truss_matches_axial_spring_solution() {
    let nodes = vec![
        Node::new_3d(0, 0.0, 1.0, 1.0),
        Node::new_3d(1, 1.0, 0.0, 1.0),
        Node::new_3d(2, 1.0, 1.0, 0.0),
        Node::new_3d(3, 1.0, 1.0, 1.0),
    ];

    let material = Material::new(200.0e9, 0.3);
    let section = Section::truss(0.01);
    let elements = vec![
        Truss3D::new(0, 0, 3, material.clone(), section.clone()),
        Truss3D::new(1, 1, 3, material.clone(), section.clone()),
        Truss3D::new(2, 2, 3, material, section),
    ];

    let mut supports = Support::pin_3d(0);
    supports.extend(Support::pin_3d(1));
    supports.extend(Support::pin_3d(2));

    let loads = vec![
        NodalLoad::new(3, Dof::Ux, 1_000.0),
        NodalLoad::new(3, Dof::Uy, 2_000.0),
        NodalLoad::new(3, Dof::Uz, -3_000.0),
    ];

    let results = linear_static::run_truss3d(&nodes, &elements, &supports, &loads)
        .expect("3D truss analysis should succeed");

    assert_close(displacement(&results, 3, Dof::Ux), 5.0e-7, 1e-15, "Ux");
    assert_close(displacement(&results, 3, Dof::Uy), 1.0e-6, 1e-15, "Uy");
    assert_close(displacement(&results, 3, Dof::Uz), -1.5e-6, 1e-15, "Uz");

    assert_close(results.member_forces[0], 1_000.0, 1e-9, "X-bar force");
    assert_close(results.member_forces[1], 2_000.0, 1e-9, "Y-bar force");
    assert_close(results.member_forces[2], -3_000.0, 1e-9, "Z-bar force");

    assert_close(
        reaction(&results, 0, Dof::Ux),
        -1_000.0,
        1e-9,
        "X support reaction",
    );
    assert_close(
        reaction(&results, 1, Dof::Uy),
        -2_000.0,
        1e-9,
        "Y support reaction",
    );
    assert_close(
        reaction(&results, 2, Dof::Uz),
        3_000.0,
        1e-9,
        "Z support reaction",
    );
}
