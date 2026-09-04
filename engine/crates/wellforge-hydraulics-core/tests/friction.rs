//! Deterministic hydraulics sanity test against the canonical fixture.

use wellforge_hydraulics_core::solve_hydraulics;
use wellforge_hydraulics_fixtures::canonical_bingham_case;

#[test]
fn bingham_case_produces_positive_losses_and_reasonable_ecd() {
    let request = canonical_bingham_case();
    let result = solve_hydraulics(&request).expect("solver must succeed");

    // For a positive flow rate we expect strictly positive pressure drops.
    assert!(result.total_pipe_pressure_loss_pa > 0.0);
    assert!(result.total_annulus_pressure_loss_pa > 0.0);
    assert!(result.bit_pressure_loss_pa > 0.0);

    // ECD must be within a physically sensible band for a 1200 kg/m^3 mud.
    let ecd = result.equivalent_circulating_density_kg_m3;
    assert!((1150.0..=1500.0).contains(&ecd), "ECD out of band: {ecd}");

    // Each section should emit exactly two records (pipe + annulus).
    assert_eq!(result.sections.len(), request.sections.len() * 2);
}
