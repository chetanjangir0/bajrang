// Integration test: full pipeline from model definition → solve → results.
// Tests the assembler, boundary condition application, and solver together.

use bajrang_core::analysis::linear_static;
use model::{
    boundary::Support, dof::Dof, elements::truss2d::Truss2D, load::NodalLoad, material::Material,
    node::Node, section::Section,
};

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
    let u_1x = results.displacements[1 * 3 + Dof::Ux as usize];
    assert!(
        (u_1x - 5e-6).abs() < 1e-12,
        "Node 1 X-displacement: expected 5e-6 m, got {:.6e} m",
        u_1x
    );

    // Node 0 and Node 1 should have zero Y displacement
    let u_0y = results.displacements[0 * 3 + Dof::Uy as usize];
    let u_1y = results.displacements[1 * 3 + Dof::Uy as usize];
    assert!(u_0y.abs() < 1e-12, "Node 0 Y should be 0, got {:.2e}", u_0y);
    assert!(u_1y.abs() < 1e-12, "Node 1 Y should be 0, got {:.2e}", u_1y);

    // Member force must equal the applied load (pure axial tension)
    assert!(
        (results.member_forces[0] - 10_000.0).abs() < 1e-6,
        "Member force: expected 10000 N, got {:.4} N",
        results.member_forces[0]
    );
}
