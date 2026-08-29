//! WITSML source identity acceptance tests.

use wellforge_witsml::{IdentityError, SourceObjectRef, WitsmlObjectType};

#[test]
fn witsml_reference_requires_uuid_and_supported_type() {
    let result = SourceObjectRef::new("not-a-uuid", WitsmlObjectType::Tubular, None);
    assert!(matches!(result, Err(IdentityError::InvalidUuid(_))));
}

#[test]
fn constructor_rejects_nil_uuid() {
    let result = SourceObjectRef::new(
        "00000000-0000-0000-0000-000000000000",
        WitsmlObjectType::Tubular,
        None,
    );
    assert!(matches!(result, Err(IdentityError::NilUuid)));
}

#[test]
fn rejects_non_absolute_uri() {
    let result = SourceObjectRef::new(
        "f3b4bb2a-e154-5dce-8f25-9c035920b20d",
        WitsmlObjectType::Trajectory,
        Some("trajectory/one"),
    );
    assert!(matches!(result, Err(IdentityError::InvalidUri(_))));
}

#[test]
fn rejects_malformed_uri_with_an_allowed_scheme_prefix() {
    let result = SourceObjectRef::new(
        "f3b4bb2a-e154-5dce-8f25-9c035920b20d",
        WitsmlObjectType::Trajectory,
        Some("https://[broken"),
    );
    assert!(matches!(result, Err(IdentityError::InvalidUri(_))));
}

#[test]
fn accepts_each_supported_absolute_uri_scheme() {
    for uri in [
        "eml:///wellforge/trajectory/one",
        "http://example.test/trajectory/one",
        "https://example.test/trajectory/one",
    ] {
        let result = SourceObjectRef::new(
            "f3b4bb2a-e154-5dce-8f25-9c035920b20d",
            WitsmlObjectType::Trajectory,
            Some(uri),
        );
        assert!(result.is_ok(), "expected {uri} to be accepted");
    }
}

#[test]
fn rejects_unsupported_absolute_uri_scheme() {
    let result = SourceObjectRef::new(
        "f3b4bb2a-e154-5dce-8f25-9c035920b20d",
        WitsmlObjectType::Trajectory,
        Some("ftp://example.test/trajectory/one"),
    );
    assert!(matches!(result, Err(IdentityError::InvalidUri(_))));
}
