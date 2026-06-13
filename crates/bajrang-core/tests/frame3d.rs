use bajrang_core::analysis::linear_static;
use model::{
    boundary::Support,
    dof::{Dof, global_dof_index},
    elements::frame3d::Frame3D,
    load::{DistributedLoad, NodalLoad},
    material::Material,
    node::Node,
    section::Section,
};

fn displacement(results: &linear_static::Frame3DResults, node: usize, dof: Dof) -> f64 {
    results.displacements[global_dof_index(node, dof)]
}

fn reaction(results: &linear_static::Frame3DResults, node: usize, dof: Dof) -> f64 {
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
fn cantilever_tip_loads_match_axial_and_biaxial_beam_solutions() {
    let nodes = vec![
        Node::new_3d(0, 0.0, 0.0, 0.0),
        Node::new_3d(1, 2.0, 0.0, 0.0),
    ];
    let material = Material::new(210.0e9, 0.3);
    let section = Section::new_3d(0.02, 4.0e-6, 8.0e-6, 1.0e-5);
    let elements = vec![Frame3D::new(0, 0, 1, material, section)];

    let supports = Support::fixed_3d(0);
    let loads = vec![
        NodalLoad::new(1, Dof::Ux, 1_000.0),
        NodalLoad::new(1, Dof::Uy, -1_000.0),
        NodalLoad::new(1, Dof::Uz, 500.0),
    ];

    let results = linear_static::run_frame3d(&nodes, &elements, &supports, &loads, &[])
        .expect("Frame3D analysis should succeed");

    assert_close(
        displacement(&results, 1, Dof::Ux),
        4.761904761904762e-7,
        1e-15,
        "Tip axial displacement",
    );
    assert_close(
        displacement(&results, 1, Dof::Uy),
        -0.0015873015873015873,
        1e-15,
        "Tip Y displacement",
    );
    assert_close(
        displacement(&results, 1, Dof::Rz),
        -0.0011904761904761906,
        1e-15,
        "Tip Z-axis rotation",
    );
    assert_close(
        displacement(&results, 1, Dof::Uz),
        0.0015873015873015873,
        1e-15,
        "Tip Z displacement",
    );
    assert_close(
        displacement(&results, 1, Dof::Ry),
        -0.0011904761904761906,
        1e-15,
        "Tip Y-axis rotation",
    );

    assert_close(reaction(&results, 0, Dof::Ux), -1_000.0, 1e-6, "X reaction");
    assert_close(reaction(&results, 0, Dof::Uy), 1_000.0, 1e-6, "Y reaction");
    assert_close(reaction(&results, 0, Dof::Uz), -500.0, 1e-6, "Z reaction");
    assert_close(
        reaction(&results, 0, Dof::Ry),
        1_000.0,
        1e-6,
        "Y moment reaction",
    );
    assert_close(
        reaction(&results, 0, Dof::Rz),
        2_000.0,
        1e-6,
        "Z moment reaction",
    );
}

#[test]
fn horizontal_member_global_y_uniform_load_produces_fixed_end_reactions() {
    let nodes = vec![
        Node::new_3d(0, 0.0, 0.0, 0.0),
        Node::new_3d(1, 3.0, 0.0, 0.0),
    ];
    let material = Material::new(210.0e9, 0.3);
    let section = Section::new_3d(0.02, 4.0e-6, 8.0e-6, 1.0e-5);
    let elements = vec![Frame3D::new(0, 0, 1, material, section)];

    let supports = Support::fixed_3d(0);
    let loads = vec![DistributedLoad::global_y(0, -2_000.0)];

    let results = linear_static::run_frame3d(&nodes, &elements, &supports, &[], &loads)
        .expect("Frame3D analysis with distributed load should succeed");

    assert_close(reaction(&results, 0, Dof::Uy), 6_000.0, 1e-6, "Y reaction");
    assert_close(
        reaction(&results, 0, Dof::Rz),
        9_000.0,
        1e-5,
        "Z moment reaction",
    );
}
