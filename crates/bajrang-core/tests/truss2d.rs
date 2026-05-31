// Integration test: full pipeline from model definition → solve → results.
// Tests the assembler, boundary condition application, and solver together.

use bajrang_core::analysis::linear_static;
use model::{
    boundary::Support, dof::Dof, elements::truss2d::Truss2D, load::NodalLoad, material::Material,
    node::Node, section::Section,
};

fn displacement(results: &linear_static::LinearStaticResults, node: usize, dof: Dof) -> f64 {
    results.displacements[node * 3 + dof as usize]
}

fn assert_close(actual: f64, expected: f64, tol: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= tol,
        "{label}: expected {expected:.12e}, got {actual:.12e}"
    );
}

#[test]
fn single_horizontal_bar_axial_load() {
    // Single horizontal bar: Node 0 at origin, Node 1 at (1m, 0)
    let nodes = vec![Node::new(0, 0.0, 0.0), Node::new(1, 1.0, 0.0)];

    let mat = Material::steel(); // E = 200 GPa
    let sec = Section::truss(0.01); // A = 0.01 m²

    let elements = vec![Truss2D::new(0, 0, 1, mat, sec)];

    // Pin at node 0 (fixes Ux, Uy), roller at node 1 (fixes Uy only)
    let mut supports = Support::pin(0);
    supports.extend(Support::roller_y(1));

    // 10 kN in +X direction at node 1
    let loads = vec![NodalLoad::new(1, Dof::Ux, 10_000.0)];

    let results =
        linear_static::run(&nodes, &elements, &supports, &loads).expect("Analysis should succeed");

    // Expected: u_1x = F / (AE/L) = 10_000 / (200e9 * 0.01 / 1.0) = 5e-6 m
    let u_1x = displacement(&results, 1, Dof::Ux);
    assert_close(u_1x, 5e-6, 1e-12, "Node 1 X-displacement");

    // Node 0 and Node 1 should have zero Y displacement
    let u_0y = displacement(&results, 0, Dof::Uy);
    let u_1y = displacement(&results, 1, Dof::Uy);
    assert_close(u_0y, 0.0, 1e-12, "Node 0 Y-displacement");
    assert_close(u_1y, 0.0, 1e-12, "Node 1 Y-displacement");

    // Member force must equal the applied load (pure axial tension)
    assert_close(results.member_forces[0], 10_000.0, 1e-6, "Member force");
}

#[test]
fn five_bar_truss() {
    let nodes = vec![
        Node::new(0, 0.0, 0.0),
        Node::new(1, 500.0, 0.0),
        Node::new(2, 300.0, 300.0),
        Node::new(3, 600.0, 300.0),
    ];

    let mat = Material::new(210_000.0, 0.3); // MPa = N/mm^2
    let sec = Section::truss(24.0); // mm^2

    let elements = vec![
        Truss2D::new(0, 0, 1, mat.clone(), sec.clone()),
        Truss2D::new(1, 0, 2, mat.clone(), sec.clone()),
        Truss2D::new(2, 1, 2, mat.clone(), sec.clone()),
        Truss2D::new(3, 1, 3, mat.clone(), sec.clone()),
        Truss2D::new(4, 2, 3, mat, sec),
    ];

    let mut supports = Support::pin(0);
    supports.push(Support::new(1, Dof::Uy));

    let loads = vec![NodalLoad::new(3, Dof::Uy, -10_000.0)];

    let results =
        linear_static::run(&nodes, &elements, &supports, &loads).expect("Analysis should succeed");

    assert_close(displacement(&results, 0, Dof::Ux), 0.0, 1e-12, "Node 1 Ux");
    assert_close(displacement(&results, 0, Dof::Uy), 0.0, 1e-12, "Node 1 Uy");
    assert_close(
        displacement(&results, 1, Dof::Ux),
        -0.1984126984,
        1e-9,
        "Node 2 Ux",
    );
    assert_close(displacement(&results, 1, Dof::Uy), 0.0, 1e-12, "Node 2 Uy");
    assert_close(
        displacement(&results, 2, Dof::Ux),
        0.2466658702,
        1e-9,
        "Node 3 Ux",
    );
    assert_close(
        displacement(&results, 2, Dof::Uy),
        0.0900516446,
        1e-9,
        "Node 3 Uy",
    );
    assert_close(
        displacement(&results, 3, Dof::Ux),
        0.4450785686,
        1e-9,
        "Node 4 Ux",
    );
    assert_close(
        displacement(&results, 3, Dof::Uy),
        -0.9116482487,
        1e-9,
        "Node 4 Uy",
    );
}

#[test]
fn three_bar_truss() {
    let nodes = vec![
        Node::new(0, 0.0, 0.0),
        Node::new(1, 144.0, 0.0),
        Node::new(2, 168.0, 0.0),
        Node::new(3, 72.0, 96.0),
    ];

    let mat = Material::new(3_000.0, 0.3); // ksi
    let sec_large = Section::truss(10.0); // in^2
    let sec_small = Section::truss(5.0); // in^2

    let elements = vec![
        Truss2D::new(0, 0, 3, mat.clone(), sec_large),
        Truss2D::new(1, 1, 3, mat.clone(), sec_small.clone()),
        Truss2D::new(2, 2, 3, mat, sec_small),
    ];

    let mut supports = Support::pin(0);
    supports.extend(Support::pin(1));
    supports.extend(Support::pin(2));

    let loads = vec![
        NodalLoad::new(3, Dof::Ux, 100.0),
        NodalLoad::new(3, Dof::Uy, -50.0),
    ];

    let results =
        linear_static::run(&nodes, &elements, &supports, &loads).expect("Analysis should succeed");

    assert_close(
        displacement(&results, 3, Dof::Ux),
        0.5300927771,
        1e-9,
        "Node 4 Ux",
    );
    assert_close(
        displacement(&results, 3, Dof::Uy),
        -0.1778936385,
        1e-9,
        "Node 4 Uy",
    );

    assert_close(
        results.member_forces[0],
        43.935188992,
        1e-6,
        "Element 1 axial force",
    );
    assert_close(
        results.member_forces[1],
        -57.546321774,
        1e-6,
        "Element 2 axial force",
    );
    assert_close(
        results.member_forces[2],
        -55.311438925,
        1e-6,
        "Element 3 axial force",
    );
}
