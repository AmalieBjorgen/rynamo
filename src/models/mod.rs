//! Data models for Dataverse metadata

mod entity;
mod solution;
mod odata;

pub use entity::{AttributeMetadata, EntityMetadata, RelationshipMetadata, LocalizedLabel};
pub use solution::{Solution, SolutionComponent, ComponentType};
pub use odata::ODataResponse;
