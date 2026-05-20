use model::elements::truss2d::Truss2D;
use model::node::Node;
use nalgebra::SMatrix;

/// 4×4 local+global stiffness matrix for a 2D truss element.
pub type Truss2DMatrix = SMatrix<f64, 4, 4>;

/// Compute the global stiffness matrix for a 2D truss element.
///
/// The formula is:
///   k = (AE/L) * T^T * k_local * T
///
/// where k_local is the axial stiffness in the element's local axis,
/// and T is the transformation matrix using direction cosines.
///
/// This returns the 4×4 matrix in global coordinates,
/// ready to be scattered into the global stiffness matrix.
pub fn stiffness_matrix(element: &Truss2D, ni: &Node, nj: &Node) -> Truss2DMatrix {
    let geom = element.geometry(ni, nj);
    let ae_over_l = element.material.elastic_modulus
        * element.section.area
        / geom.length;

    let c = geom.cos;
    let s = geom.sin;

    // Direct formulation of the 4×4 global stiffness:
    // k = (AE/L) * [c², cs, -c², -cs; 
    //               cs, s², -cs, -s²; 
    //               -c², -cs, c², cs; 
    //               -cs, -s², cs, s²]
    #[rustfmt::skip]
    let k = Truss2DMatrix::from_row_slice(&[
         c*c,  c*s, -c*c, -c*s,
         c*s,  s*s, -c*s, -s*s,
        -c*c, -c*s,  c*c,  c*s,
        -c*s, -s*s,  c*s,  s*s,
    ]);

    ae_over_l * k
}

/// Recover the axial force in the element after solving displacements.
///
/// Positive = tension, Negative = compression.
pub fn axial_force(
    element: &Truss2D,
    ni: &Node,
    nj: &Node,
    displacements: &[f64],
) -> f64 {
    let geom = element.geometry(ni, nj);
    let ae_over_l = element.material.elastic_modulus
        * element.section.area
        / geom.length;

    let c = geom.cos;
    let s = geom.sin;

    let dofs = element.global_dof_indices();
    let u_ix = displacements[dofs[0]];
    let u_iy = displacements[dofs[1]];
    let u_jx = displacements[dofs[2]];
    let u_jy = displacements[dofs[3]];

    // Axial elongation projected onto element axis
    ae_over_l * (c * (u_jx - u_ix) + s * (u_jy - u_iy))
}
