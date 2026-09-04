//! Bottom-hole-assembly mechanics with independent catalog and beam modules.

mod beam;
mod catalog;

pub use beam::{BeamElement, BeamError, BeamResponse};
pub use catalog::{BhaAssembly, BhaComponent, BhaComponentKind, CatalogError};
