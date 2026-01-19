//! Entity and attribute metadata API

use super::DataverseClient;
use crate::models::{AttributeMetadata, EntityMetadata, ODataResponse, RelationshipMetadata};
use anyhow::Result;

impl DataverseClient {
    /// Get all entity definitions
    pub async fn get_entities(&self) -> Result<Vec<EntityMetadata>> {
        let response: ODataResponse<EntityMetadata> = self
            .get_json("EntityDefinitions?$select=LogicalName,DisplayName,SchemaName,Description,PrimaryIdAttribute,PrimaryNameAttribute,EntitySetName,IsCustomEntity,IsManaged,ObjectTypeCode")
            .await?;
        Ok(response.value)
    }

    /// Get a specific entity by logical name
    pub async fn get_entity(&self, logical_name: &str) -> Result<EntityMetadata> {
        let endpoint = format!(
            "EntityDefinitions(LogicalName='{}')?$select=LogicalName,DisplayName,SchemaName,Description,PrimaryIdAttribute,PrimaryNameAttribute,EntitySetName,IsCustomEntity,IsManaged,ObjectTypeCode",
            logical_name
        );
        self.get_json(&endpoint).await
    }

    /// Get attributes for an entity
    pub async fn get_entity_attributes(&self, logical_name: &str) -> Result<Vec<AttributeMetadata>> {
        let endpoint = format!(
            "EntityDefinitions(LogicalName='{}')/Attributes?$select=LogicalName,DisplayName,SchemaName,AttributeType,AttributeTypeName,RequiredLevel,IsCustomAttribute,IsPrimaryId,IsPrimaryName,Description",
            logical_name
        );
        let response: ODataResponse<AttributeMetadata> = self.get_json(&endpoint).await?;
        Ok(response.value)
    }

    /// Get relationships for an entity (1:N)
    pub async fn get_entity_one_to_many(&self, logical_name: &str) -> Result<Vec<RelationshipMetadata>> {
        let endpoint = format!(
            "EntityDefinitions(LogicalName='{}')/OneToManyRelationships?$select=SchemaName,ReferencingEntity,ReferencingAttribute,ReferencedEntity,ReferencedAttribute",
            logical_name
        );
        let response: ODataResponse<RelationshipMetadata> = self.get_json(&endpoint).await?;
        Ok(response.value)
    }

    /// Get relationships for an entity (N:1)
    pub async fn get_entity_many_to_one(&self, logical_name: &str) -> Result<Vec<RelationshipMetadata>> {
        let endpoint = format!(
            "EntityDefinitions(LogicalName='{}')/ManyToOneRelationships?$select=SchemaName,ReferencingEntity,ReferencingAttribute,ReferencedEntity,ReferencedAttribute",
            logical_name
        );
        let response: ODataResponse<RelationshipMetadata> = self.get_json(&endpoint).await?;
        Ok(response.value)
    }

    /// Get N:N relationships for an entity
    pub async fn get_entity_many_to_many(&self, logical_name: &str) -> Result<Vec<RelationshipMetadata>> {
        let endpoint = format!(
            "EntityDefinitions(LogicalName='{}')/ManyToManyRelationships?$select=SchemaName,Entity1LogicalName,Entity2LogicalName,IntersectEntityName",
            logical_name
        );
        let response: ODataResponse<RelationshipMetadata> = self.get_json(&endpoint).await?;
        Ok(response.value)
    }
}
