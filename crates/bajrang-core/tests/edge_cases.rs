use bajrang_core::{
    analysis::linear_static::{self, AnalysisError},
    solver::SolverError,
};
use model::{
    boundary::Support,
    dof::{Dof, global_dof_index},
    elements::{beam2d::Beam2D, frame2d::Frame2D, truss2d::Truss2D},
    load::{DistributedLoad, NodalLoad},
    material::Material,
    node::Node,
    section::Section,
};

fn displacement(values: &[f64], node: usize, dof: Dof) -> f64 {
    values[global_dof_index(node, dof)]
}

fn assert_close(actual: f64, expected: f64, tol: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= tol,
        "{label}: expected {expected:.12e}, got {actual:.12e}"
    );
}

#[test]
fn underconstrained_truss_returns_singular_matrix_error() {
    let nodes = vec![Node::new(0, 0.0, 0.0), Node::new(1, 1.0, 0.0)];
    let elements = vec![Truss2D::new(
        0,
        0,
        1,
        Material::steel(),
        Section::truss(0.01),
    )];
    let supports = vec![Support::new(0, Dof::Ux)];
    let loads = vec![NodalLoad::new(1, Dof::Ux, 10_000.0)];

    let err = linear_static::run(&nodes, &elements, &supports, &loads)
        .expect_err("Model should remain singular due to rigid-body translation in Y");

    match err {
        AnalysisError::Solver(SolverError::SingularMatrix) => {}
    }
}

#[test]
fn frame_vertical_member_axial_tip_load_matches_bar_solution() {
    let nodes = vec![Node::new(0, 0.0, 0.0), Node::new(1, 0.0, 3.0)];
    let elements = vec![Frame2D::new(
        0,
        0,
        1,
        Material::new(210.0e9, 0.3),
        Section::new(0.02, 1.6e-5),
    )];
    let supports = vec![
        Support::new(0, Dof::Ux),
        Support::new(0, Dof::Uy),
        Support::new(0, Dof::Rz),
    ];
    let loads = vec![NodalLoad::new(1, Dof::Uy, -50_000.0)];

    let results = linear_static::run_frame2d(&nodes, &elements, &supports, &loads, &[])
        .expect("Frame analysis should succeed");

    assert_close(
        displacement(&results.displacements, 1, Dof::Ux),
        0.0,
        1e-12,
        "Tip horizontal displacement",
    );
    assert_close(
        displacement(&results.displacements, 1, Dof::Uy),
        -3.5714285714285714e-5,
        1e-15,
        "Tip axial shortening",
    );
    assert_close(
        displacement(&results.displacements, 1, Dof::Rz),
        0.0,
        1e-12,
        "Tip rotation",
    );

    let end_forces = results.member_end_forces[0];
    assert_close(end_forces[0], 50_000.0, 1e-6, "Node i axial force");
    assert_close(end_forces[1], 0.0, 1e-6, "Node i shear");
    assert_close(end_forces[2], 0.0, 1e-6, "Node i moment");
    assert_close(end_forces[3], -50_000.0, 1e-6, "Node j axial force");
    assert_close(end_forces[4], 0.0, 1e-6, "Node j shear");
    assert_close(end_forces[5], 0.0, 1e-6, "Node j moment");
}

#[test]
fn distributed_loads_on_other_members_do_not_pollute_beam_solution() {
    let nodes = vec![Node::new(0, 0.0, 0.0), Node::new(1, 2.0, 0.0)];
    let elements = vec![Beam2D::new(
        0,
        0,
        1,
        Material::new(200.0e9, 0.3),
        Section::new(0.02, 8.0e-6),
    )];
    let supports = vec![Support::new(0, Dof::Uy), Support::new(0, Dof::Rz)];
    let loads = vec![NodalLoad::new(1, Dof::Uy, -1_000.0)];
    let unrelated_distributed = vec![DistributedLoad::local_y(99, -1_000.0)];

    let results =
        linear_static::run_beam2d(&nodes, &elements, &supports, &loads, &unrelated_distributed)
            .expect("Beam analysis should ignore unrelated distributed loads");

    assert_close(
        displacement(&results.displacements, 1, Dof::Uy),
        -1.0 / 600.0,
        1e-12,
        "Tip deflection",
    );
    assert_close(
        displacement(&results.displacements, 1, Dof::Rz),
        -0.00125,
        1e-12,
        "Tip rotation",
    );
}
