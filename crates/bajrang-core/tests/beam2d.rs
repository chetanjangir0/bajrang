use bajrang_core::analysis::linear_static;
use model::{
    boundary::Support,
    dof::{Dof, global_dof_index},
    elements::beam2d::Beam2D,
    load::{DistributedLoad, NodalLoad},
    material::Material,
    node::Node,
    section::Section,
};

fn displacement(results: &linear_static::Beam2DResults, node: usize, dof: Dof) -> f64 {
    results.displacements[global_dof_index(node, dof)]
}

fn reaction(results: &linear_static::Beam2DResults, node: usize, dof: Dof) -> f64 {
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
fn cantilever_tip_load_matches_euler_bernoulli_solution() {
    let nodes = vec![Node::new(0, 0.0, 0.0), Node::new(1, 2.0, 0.0)];
    let material = Material::new(200.0e9, 0.3);
    let section = Section::new(0.02, 8.0e-6);
    let elements = vec![Beam2D::new(0, 0, 1, material, section)];

    let supports = vec![Support::new(0, Dof::Uy), Support::new(0, Dof::Rz)];
    let loads = vec![NodalLoad::new(1, Dof::Uy, -1_000.0)];

    let results = linear_static::run_beam2d(&nodes, &elements, &supports, &loads, &[])
        .expect("Beam analysis should succeed");

    assert_close(
        displacement(&results, 1, Dof::Uy),
        -1.0 / 600.0,
        1e-12,
        "Tip deflection",
    );
    assert_close(
        displacement(&results, 1, Dof::Rz),
        -0.00125,
        1e-12,
        "Tip rotation",
    );

    let end_forces = results.member_end_forces[0];
    assert_close(end_forces[0], 1_000.0, 1e-6, "Node i shear");
    assert_close(end_forces[1], 2_000.0, 1e-6, "Node i moment");
    assert_close(end_forces[2], -1_000.0, 1e-6, "Node j shear");
    assert_close(end_forces[3], 0.0, 1e-6, "Node j moment");

    assert_close(
        reaction(&results, 0, Dof::Uy),
        1_000.0,
        1e-6,
        "Node 0 Y reaction",
    );
    assert_close(
        reaction(&results, 0, Dof::Rz),
        2_000.0,
        1e-6,
        "Node 0 moment reaction",
    );
}

#[test]
fn simply_supported_uniform_load_affects_rotations() {
    let nodes = vec![Node::new(0, 0.0, 0.0), Node::new(1, 2.0, 0.0)];
    let material = Material::new(200.0e9, 0.3);
    let section = Section::new(0.02, 8.0e-6);
    let elements = vec![Beam2D::new(0, 0, 1, material, section)];

    let supports = vec![Support::new(0, Dof::Uy), Support::new(1, Dof::Uy)];
    let loads = vec![];
    let distributed_loads = vec![DistributedLoad::local_y(0, -1_000.0)];

    let results =
        linear_static::run_beam2d(&nodes, &elements, &supports, &loads, &distributed_loads)
            .expect("Beam analysis with distributed load should succeed");

    assert_close(
        displacement(&results, 0, Dof::Rz),
        -2.0833333333333334e-4,
        1e-12,
        "Left rotation",
    );
    assert_close(
        displacement(&results, 1, Dof::Rz),
        2.0833333333333334e-4,
        1e-12,
        "Right rotation",
    );

    assert_close(
        reaction(&results, 0, Dof::Uy),
        1_000.0,
        1e-6,
        "Left support reaction",
    );
    assert_close(
        reaction(&results, 1, Dof::Uy),
        1_000.0,
        1e-6,
        "Right support reaction",
    );
}

#[test]
fn simply_supported_global_y_uniform_load_matches_local_y_solution() {
    let nodes = vec![Node::new(0, 0.0, 0.0), Node::new(1, 2.0, 0.0)];
    let material = Material::new(200.0e9, 0.3);
    let section = Section::new(0.02, 8.0e-6);
    let elements = vec![Beam2D::new(0, 0, 1, material, section)];

    let supports = vec![Support::new(0, Dof::Uy), Support::new(1, Dof::Uy)];
    let distributed_loads = vec![DistributedLoad::global_y(0, -1_000.0)];

    let results = linear_static::run_beam2d(&nodes, &elements, &supports, &[], &distributed_loads)
        .expect("Beam analysis with global distributed load should succeed");

    assert_close(
        displacement(&results, 0, Dof::Rz),
        -2.0833333333333334e-4,
        1e-12,
        "Left rotation",
    );
    assert_close(
        displacement(&results, 1, Dof::Rz),
        2.0833333333333334e-4,
        1e-12,
        "Right rotation",
    );

    assert_close(
        reaction(&results, 0, Dof::Uy),
        1_000.0,
        1e-6,
        "Left support reaction",
    );
    assert_close(
        reaction(&results, 1, Dof::Uy),
        1_000.0,
        1e-6,
        "Right support reaction",
    );
}
