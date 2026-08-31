#[test]
fn crate_exposes_a_typed_error() {
    let error = wellforge_bha_interchange::InterchangeError::UnsupportedRoot("Other".into());
    assert_eq!(error.to_string(), "unsupported BHA XML root: Other");
}
