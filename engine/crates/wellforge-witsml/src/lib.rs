//! WITSML 2.0-aligned source identities used by analysis contracts.

mod identity;
mod projection;

pub use identity::{IdentityError, SourceObjectRef, WitsmlObjectType};
pub use projection::{ProjectionError, project_source_identity};
