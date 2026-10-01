//! Acceptance tests proving the independent harness consumes production numerical primitives.

use wellforge_numerics_acceptance::run;

#[test]
fn production_foundational_primitives_are_exercised() {
    let report = run();

    assert!(report.production_root_solve);
    assert!(report.production_volume_round_trip);
    assert!(report.production_monotone_interpolation);
    assert!(report.production_circular_coherence);
    assert!(report.production_complementarity);
    assert!(report.production_sparse_symbolic_reuse);
}
