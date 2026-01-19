//! Data models for Dataverse metadata

mod entity;
mod solution;
mod odata;
mod user;
mod query;

pub use entity::{AttributeMetadata, EntityMetadata, RelationshipMetadata, LocalizedLabel};
pub use solution::{Solution, SolutionComponent, ComponentType};
pub use odata::ODataResponse;
pub use user::{SystemUser, Team, SecurityRole, RoleAssignment, RoleSource};
pub use query::{QueryDefinition, QueryResult, QueryField};
