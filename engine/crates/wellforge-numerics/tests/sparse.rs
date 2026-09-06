//! Contract tests for reusable sparse linear systems.

use wellforge_numerics::{SparseEntry, SparseError, SparseLinearSystem};

fn tridiagonal(diagonal: f64) -> Vec<SparseEntry> {
    vec![
        SparseEntry::new(0, 0, diagonal),
        SparseEntry::new(1, 0, -1.0),
        SparseEntry::new(0, 1, -1.0),
        SparseEntry::new(1, 1, diagonal),
        SparseEntry::new(2, 1, -1.0),
        SparseEntry::new(1, 2, -1.0),
        SparseEntry::new(2, 2, diagonal),
    ]
}

#[test]
fn sparse_system_solves_known_tridiagonal_case() {
    let entries = tridiagonal(2.0);
    let system = SparseLinearSystem::new(3, &entries).expect("valid sparse pattern");
    let solution = system
        .factor_and_solve(&entries, &[0.0, 0.0, 4.0])
        .expect("nonsingular solve");

    for (actual, expected) in solution.values.iter().zip([1.0, 2.0, 3.0]) {
        assert!((*actual - expected).abs() < 1.0e-12);
    }
    assert!(solution.residual_norm < 1.0e-12);
}

#[test]
fn sparse_system_reuses_symbolic_pattern_with_new_numeric_values() {
    let initial = tridiagonal(2.0);
    let system = SparseLinearSystem::new(3, &initial).expect("valid sparse pattern");
    let updated = tridiagonal(3.0);

    let solution = system
        .factor_and_solve(&updated, &[1.0, 2.0, 7.0])
        .expect("same pattern numeric refactorization");

    for (actual, expected) in solution.values.iter().zip([1.0, 2.0, 3.0]) {
        assert!((*actual - expected).abs() < 1.0e-12);
    }
}

#[test]
fn sparse_system_rejects_changed_pattern() {
    let entries = tridiagonal(2.0);
    let system = SparseLinearSystem::new(3, &entries).expect("valid sparse pattern");
    let mut changed = entries.clone();
    changed[0] = SparseEntry::new(0, 1, 2.0);

    let error = system
        .factor_and_solve(&changed, &[0.0, 0.0, 4.0])
        .expect_err("changed sparsity must be rejected");

    assert_eq!(error, SparseError::PatternMismatch);
}
