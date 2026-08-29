//! Modal-analysis acceptance tests.

#[test]
fn returns_positive_sorted_modes() {
    let request = wellforge_bha_fixtures::minimal_request();
    let bha_model = wellforge_bha_model::assemble_model(&request).unwrap();
    let static_solution = wellforge_bha_static::solve_static(&bha_model, &request).unwrap();
    let mode_results =
        wellforge_bha_modal::solve_modes(&bha_model, &request, &static_solution).unwrap();
    assert!(!mode_results.is_empty());
    assert!(
        mode_results
            .windows(2)
            .all(|pair| pair[0].natural_frequency_hz <= pair[1].natural_frequency_hz)
    );
    assert!(
        mode_results
            .iter()
            .all(|mode| mode.natural_frequency_hz > 0.0)
    );
}

#[test]
fn harmonic_response_peaks_near_first_mode() {
    let request = wellforge_bha_fixtures::minimal_request();
    let bha_model = wellforge_bha_model::assemble_model(&request).unwrap();
    let static_solution = wellforge_bha_static::solve_static(&bha_model, &request).unwrap();
    let mode_results =
        wellforge_bha_modal::solve_modes(&bha_model, &request, &static_solution).unwrap();
    let response = wellforge_bha_modal::solve_frequency_response(
        &static_solution,
        0.5,
        mode_results[0].natural_frequency_hz * 1.5,
        100,
    )
    .unwrap();
    let peak = response
        .iter()
        .max_by(|a, b| a.receptance_m_n.total_cmp(&b.receptance_m_n))
        .unwrap();
    assert!(
        (peak.frequency_hz - mode_results[0].natural_frequency_hz).abs()
            < mode_results[0].natural_frequency_hz * 0.1
    );
}

#[test]
fn compressive_wob_reduces_first_lateral_frequency() {
    let unloaded = wellforge_bha_fixtures::minimal_request();
    let model = wellforge_bha_model::assemble_model(&unloaded).unwrap();
    let static_unloaded = wellforge_bha_static::solve_static(&model, &unloaded).unwrap();
    let frequency_unloaded = wellforge_bha_modal::solve_modes(&model, &unloaded, &static_unloaded)
        .unwrap()[0]
        .natural_frequency_hz;
    let mut loaded = unloaded.clone();
    loaded.operating.wob_n = 50_000.0;
    let static_loaded = wellforge_bha_static::solve_static(&model, &loaded).unwrap();
    let frequency_loaded = wellforge_bha_modal::solve_modes(&model, &loaded, &static_loaded)
        .unwrap()[0]
        .natural_frequency_hz;
    assert!(frequency_loaded < frequency_unloaded);
}
