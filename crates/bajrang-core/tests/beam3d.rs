use bajrang_core::analysis::linear_static;
use model::{
    boundary::Support,
    dof::{Dof, global_dof_index},
    elements::beam3d::Beam3D,
    load::{DistributedLoad, NodalLoad},
    material::Material,
    node::Node,
    section::Section,
};

fn displacement(results: &linear_static::Beam3DResults, node: usize, dof: Dof) -> f64 {
    results.displacements[global_dof_index(node, dof)]
}

fn reaction(results: &linear_static::Beam3DResults, node: usize, dof: Dof) -> f64 {
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
fn cantilever_tip_loads_match_biaxial_euler_bernoulli_solution() {
    let nodes = vec![
        Node::new_3d(0, 0.0, 0.0, 0.0),
        Node::new_3d(1, 2.0, 0.0, 0.0),
    ];
    let material = Material::new(200.0e9, 0.3);
    let section = Section::new_3d(0.02, 4.0e-6, 8.0e-6, 1.0e-5);
    let elements = vec![Beam3D::new(0, 0, 1, material, section)];

    let supports = vec![
        Support::new(0, Dof::Uy),
        Support::new(0, Dof::Uz),
        Support::new(0, Dof::Rx),
        Support::new(0, Dof::Ry),
        Support::new(0, Dof::Rz),
    ];
    let loads = vec![
        NodalLoad::new(1, Dof::Uy, -1_000.0),
        NodalLoad::new(1, Dof::Uz, 500.0),
    ];

    let results = linear_static::run_beam3d(&nodes, &elements, &supports, &loads, &[])
        .expect("Beam3D analysis should succeed");

    assert_close(
        displacement(&results, 1, Dof::Uy),
        -1.0 / 600.0,
        1e-15,
        "Tip local-y displacement",
    );
    assert_close(
        displacement(&results, 1, Dof::Rz),
        -1.25e-3,
        1e-15,
        "Tip local-z rotation",
    );
    assert_close(
        displacement(&results, 1, Dof::Uz),
        1.0 / 600.0,
        1e-12,
        "Tip local-z displacement",
    );
    assert_close(
        displacement(&results, 1, Dof::Ry),
        -0.00125,
        1e-12,
        "Tip local-y rotation",
    );

    assert_close(
        reaction(&results, 0, Dof::Uy),
        1_000.0,
        1e-6,
        "Node 0 Y reaction",
    );
    assert_close(
        reaction(&results, 0, Dof::Uz),
        -500.0,
        1e-6,
        "Node 0 Z reaction",
    );
    assert_close(
        reaction(&results, 0, Dof::Ry),
        1_000.0,
        1e-6,
        "Node 0 local-y moment reaction",
    );
    assert_close(
        reaction(&results, 0, Dof::Rz),
        2_000.0,
        1e-6,
        "Node 0 local-z moment reaction",
    );
}

#[test]
fn simply_supported_global_z_uniform_load_matches_closed_form_reactions() {
    let nodes = vec![
        Node::new_3d(0, 0.0, 0.0, 0.0),
        Node::new_3d(1, 2.0, 0.0, 0.0),
    ];
    let material = Material::new(200.0e9, 0.3);
    let section = Section::new_3d(0.02, 4.0e-6, 8.0e-6, 1.0e-5);
    let elements = vec![Beam3D::new(0, 0, 1, material, section)];

    let supports = vec![
        Support::new(0, Dof::Uy),
        Support::new(0, Dof::Uz),
        Support::new(1, Dof::Uy),
        Support::new(1, Dof::Uz),
        Support::new(0, Dof::Rx),
        Support::new(0, Dof::Rz),
    ];
    let distributed_loads = vec![DistributedLoad::global_z(0, -1_000.0)];

    let results = linear_static::run_beam3d(&nodes, &elements, &supports, &[], &distributed_loads)
        .expect("Beam3D analysis with distributed load should succeed");

    assert_close(
        reaction(&results, 0, Dof::Uz),
        1_000.0,
        1e-6,
        "Left support reaction",
    );
    assert_close(
        reaction(&results, 1, Dof::Uz),
        1_000.0,
        1e-6,
        "Right support reaction",
    );
}
