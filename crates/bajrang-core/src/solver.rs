use nalgebra::{DMatrix, DVector};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SolverError {
    #[error("Global stiffness matrix is singular — check boundary conditions")]
    SingularMatrix,
}

/// Solve K * u = F using LU decomposition.
///
/// Requires that boundary conditions have already been applied to K and F.
/// Returns the displacement vector u.
pub fn solve(k: DMatrix<f64>, f: Vec<f64>) -> Result<Vec<f64>, SolverError> {
    let f_vec = DVector::from_vec(f);

    k.lu()
        .solve(&f_vec)
        .map(|u| u.as_slice().to_vec())
        .ok_or(SolverError::SingularMatrix)
}
