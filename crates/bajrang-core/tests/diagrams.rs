use bajrang_core::{
    analysis::linear_static,
    post::diagrams::{DiagramKind, MemberDiagram},
};
use model::{
    boundary::Support,
    dof::Dof,
    elements::{beam2d::Beam2D, frame2d::Frame2D},
    load::{DistributedLoad, NodalLoad},
    material::Material,
    node::Node,
    section::Section,
};

fn diagram(diagrams: &[MemberDiagram], element_id: usize, kind: DiagramKind) -> &MemberDiagram {
    diagrams
        .iter()
        .find(|diagram| diagram.element_id == element_id && diagram.kind == kind)
        .expect("expected member diagram")
}

fn value_at(diagram: &MemberDiagram, x: f64) -> f64 {
    diagram
        .points
        .iter()
        .find(|point| (point.x - x).abs() <= 1e-12)
        .map(|point| point.value)
        .expect("expected diagram station")
}

fn assert_close(actual: f64, expected: f64, tol: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= tol,
        "{label}: expected {expected:.12e}, got {actual:.12e}"
    );
}

#[test]
fn beam2d_cantilever_tip_load_diagrams_match_closed_form_values() {
    let nodes = vec![Node::new(0, 0.0, 0.0), Node::new(1, 2.0, 0.0)];
    let material = Material::new(200.0e9, 0.3);
    let section = Section::new(0.02, 8.0e-6);
    let elements = vec![Beam2D::new(0, 0, 1, material, section)];
    let supports = vec![Support::new(0, Dof::Uy), Support::new(0, Dof::Rz)];
    let loads = vec![NodalLoad::new(1, Dof::Uy, -1_000.0)];

    let results = linear_static::run_beam2d(&nodes, &elements, &supports, &loads, &[])
        .expect("beam analysis should succeed");
    let shear = diagram(&results.member_diagrams, 0, DiagramKind::ShearY);
    let moment = diagram(&results.member_diagrams, 0, DiagramKind::MomentZ);

    assert_close(value_at(shear, 0.0), 1_000.0, 1e-6, "Root shear");
    assert_close(value_at(shear, 2.0), 1_000.0, 1e-6, "Tip shear");
    assert_close(value_at(moment, 0.0), 2_000.0, 1e-6, "Root moment");
    assert_close(value_at(moment, 2.0), 0.0, 1e-6, "Tip moment");
}

#[test]
fn beam2d_uniform_load_diagrams_include_parabolic_bending() {
    let nodes = vec![Node::new(0, 0.0, 0.0), Node::new(1, 2.0, 0.0)];
    let material = Material::new(200.0e9, 0.3);
    let section = Section::new(0.02, 8.0e-6);
    let elements = vec![Beam2D::new(0, 0, 1, material, section)];
    let supports = vec![Support::new(0, Dof::Uy), Support::new(1, Dof::Uy)];
    let distributed_loads = vec![DistributedLoad::local_y(0, -1_000.0)];

    let results = linear_static::run_beam2d(&nodes, &elements, &supports, &[], &distributed_loads)
        .expect("beam analysis should succeed");
    let shear = diagram(&results.member_diagrams, 0, DiagramKind::ShearY);
    let moment = diagram(&results.member_diagrams, 0, DiagramKind::MomentZ);

    assert_close(value_at(shear, 0.0), 1_000.0, 1e-6, "Left shear");
    assert_close(value_at(shear, 1.0), 0.0, 1e-6, "Midspan shear");
    assert_close(value_at(shear, 2.0), -1_000.0, 1e-6, "Right shear");
    assert_close(value_at(moment, 0.0), 0.0, 1e-6, "Left moment");
    assert_close(value_at(moment, 1.0), -500.0, 1e-6, "Midspan moment");
    assert_close(value_at(moment, 2.0), 0.0, 1e-6, "Right moment");
}

#[test]
fn frame2d_global_load_diagrams_use_local_member_coordinates() {
    let nodes = vec![Node::new(0, 0.0, 0.0), Node::new(1, 0.0, 3.0)];
    let material = Material::new(210.0e9, 0.3);
    let section = Section::new(0.02, 1.6e-5);
    let elements = vec![Frame2D::new(0, 0, 1, material, section)];
    let supports = vec![
        Support::new(0, Dof::Ux),
        Support::new(0, Dof::Uy),
        Support::new(0, Dof::Rz),
    ];
    let distributed_loads = vec![DistributedLoad::global_x(0, 2_000.0)];

    let results = linear_static::run_frame2d(&nodes, &elements, &supports, &[], &distributed_loads)
        .expect("frame analysis should succeed");
    let shear = diagram(&results.member_diagrams, 0, DiagramKind::ShearY);
    let moment = diagram(&results.member_diagrams, 0, DiagramKind::MomentZ);

    assert_close(value_at(shear, 0.0), 6_000.0, 1e-6, "Root shear");
    assert_close(value_at(shear, 3.0), 0.0, 1e-6, "Tip shear");
    assert_close(value_at(moment, 0.0), 9_000.0, 1e-5, "Root moment");
    assert_close(value_at(moment, 3.0), 0.0, 1e-5, "Tip moment");
}
