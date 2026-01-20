//! Data models for Dataverse metadata

mod entity;
mod solution;
mod odata;
mod user;
mod query;

pub use entity::{AttributeMetadata, EntityMetadata, RelationshipMetadata, OptionSetMetadata};
pub use solution::{Solution, SolutionComponent, ComponentType, SolutionComponentLayer};
pub use odata::ODataResponse;
pub use user::{SystemUser, Team, SecurityRole, RoleAssignment, RoleSource};
pub mod discovery;

pub use discovery::{DiscoveryInstance, DiscoveryResponse};
pub use query::QueryResult;
