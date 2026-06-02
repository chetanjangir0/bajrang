use bajrang_core::analysis::linear_static::{self, ElementResult};
use model::{
    boundary::Support,
    dof::Dof,
    elements::{StructuralElement, beam2d::Beam2D, frame2d::Frame2D, truss2d::Truss2D},
    load::NodalLoad,
    material::Material,
    node::Node,
    section::Section,
};

fn displacement(results: &linear_static::Mixed2DResults, node: usize, dof: Dof) -> f64 {
    results.displacements[node * 3 + dof as usize]
}

fn reaction(results: &linear_static::Mixed2DResults, node: usize, dof: Dof) -> f64 {
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
fn mixed_beam_and_truss_share_one_solution() {
    let nodes = vec![Node::new(0, 0.0, 0.0), Node::new(1, 2.0, 0.0)];

    let truss = Truss2D::new(0, 0, 1, Material::new(200.0e9, 0.3), Section::truss(0.01));
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

    assert_close(
        displacement(&results, 1, Dof::Ux),
        1.0e-5,
        1e-12,
        "Node 1 Ux",
    );
    assert_close(
        displacement(&results, 1, Dof::Uy),
        -1.0 / 600.0,
        1e-12,
        "Node 1 Uy",
    );
    assert_close(
        displacement(&results, 1, Dof::Rz),
        -0.00125,
        1e-12,
        "Node 1 Rz",
    );

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

    assert_close(
        reaction(&results, 0, Dof::Ux),
        -10_000.0,
        1e-6,
        "Node 0 X reaction",
    );
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
fn calfem_frame_and_bars_mixed_system_matches_reference_displacements() {
    // Reference:
    // CALFEM for Python, "Example: Frame and bars" (exs7)
    // https://calfem-for-python.readthedocs.io/en/latest/examples/exs7.html
    let nodes = vec![
        Node::new(0, 0.0, 0.0),
        Node::new(1, 1.0, 0.0),
        Node::new(2, 0.0, 1.0),
        Node::new(3, 1.0, 1.0),
        Node::new(4, 0.0, 2.0),
        Node::new(5, 1.0, 2.0),
    ];

    let material = Material::new(1.0, 0.3);
    let frame_section = Section::new(1.0, 1.0);
    let truss_section = Section::truss(1.0);

    let elements = vec![
        StructuralElement::Frame2D(Frame2D::new(
            0,
            0,
            2,
            material.clone(),
            frame_section.clone(),
        )),
        StructuralElement::Frame2D(Frame2D::new(
            1,
            2,
            4,
            material.clone(),
            frame_section.clone(),
        )),
        StructuralElement::Frame2D(Frame2D::new(
            2,
            1,
            3,
            material.clone(),
            frame_section.clone(),
        )),
        StructuralElement::Frame2D(Frame2D::new(
            3,
            3,
            5,
            material.clone(),
            frame_section.clone(),
        )),
        StructuralElement::Frame2D(Frame2D::new(
            4,
            2,
            3,
            material.clone(),
            frame_section.clone(),
        )),
        StructuralElement::Frame2D(Frame2D::new(
            5,
            4,
            5,
            material.clone(),
            frame_section.clone(),
        )),
        StructuralElement::Truss2D(Truss2D::new(
            6,
            0,
            3,
            material.clone(),
            truss_section.clone(),
        )),
        StructuralElement::Truss2D(Truss2D::new(
            7,
            2,
            5,
            material.clone(),
            truss_section.clone(),
        )),
        StructuralElement::Truss2D(Truss2D::new(
            8,
            2,
            1,
            material.clone(),
            truss_section.clone(),
        )),
        StructuralElement::Truss2D(Truss2D::new(9, 4, 3, material, truss_section)),
    ];

    let supports = vec![
        Support::new(0, Dof::Ux),
        Support::new(0, Dof::Uy),
        Support::new(0, Dof::Rz),
        Support::new(1, Dof::Ux),
        Support::new(1, Dof::Uy),
        Support::new(1, Dof::Rz),
    ];
    let loads = vec![NodalLoad::new(4, Dof::Ux, 1.0)];

    let results = linear_static::run_mixed(&nodes, &elements, &supports, &loads, &[])
        .expect("Mixed analysis should succeed");

    let expected_displacements = [
        (2, Dof::Ux, 0.37905924),
        (2, Dof::Uy, 0.30451926),
        (2, Dof::Rz, -0.65956297),
        (3, Dof::Ux, 0.30414480),
        (3, Dof::Uy, -0.28495132),
        (3, Dof::Rz, -0.54570174),
        (4, Dof::Ux, 1.19791809),
        (4, Dof::Uy, 0.44655174),
        (4, Dof::Rz, -0.85908643),
        (5, Dof::Ux, 0.96969909),
        (5, Dof::Uy, -0.34780417),
        (5, Dof::Rz, -0.74373562),
    ];

    for (node_id, dof, expected) in expected_displacements {
        assert_close(
            displacement(&results, node_id, dof),
            expected,
            1e-8,
            "CALFEM mixed displacement",
        );
    }
}
