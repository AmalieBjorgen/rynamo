//! Entity and attribute metadata API

use super::DataverseClient;
use crate::models::{AttributeMetadata, EntityMetadata, RelationshipMetadata, OptionSetMetadata};
use crate::models::odata::ODataResponse;
use anyhow::Result;

impl DataverseClient {
    /// Get all global option sets
    pub async fn get_global_option_sets(&self) -> Result<Vec<OptionSetMetadata>> {
        let response: ODataResponse<OptionSetMetadata> = self
            .get_json("GlobalOptionSetDefinitions?$select=Name,DisplayName,Description,IsGlobal,OptionSetType,MetadataId")
            .await?;
        Ok(response.value)
    }

    /// Get the OptionSet for a specific attribute
    pub async fn get_attribute_option_set(&self, entity_logical_name: &str, attribute_logical_name: &str) -> Result<OptionSetMetadata> {
        let endpoint = format!(
            "EntityDefinitions(LogicalName='{}')/Attributes(LogicalName='{}')/Microsoft.Dynamics.CRM.PicklistAttributeMetadata/OptionSet?$select=Name,DisplayName,Description,IsGlobal,OptionSetType,MetadataId",
            entity_logical_name, attribute_logical_name
        );
        self.get_json(&endpoint).await
    }

    /// Get all entity definitions
    pub async fn get_entities(&self) -> Result<Vec<EntityMetadata>> {
        let response: ODataResponse<EntityMetadata> = self
            .get_json("EntityDefinitions?$select=LogicalName,DisplayName,SchemaName,Description,PrimaryIdAttribute,PrimaryNameAttribute,EntitySetName,IsCustomEntity,IsManaged,ObjectTypeCode,MetadataId")
            .await?;
        Ok(response.value)
    }

    /// Get a specific entity by logical name
    pub async fn get_entity(&self, logical_name: &str) -> Result<EntityMetadata> {
        let endpoint = format!(
            "EntityDefinitions(LogicalName='{}')?$select=LogicalName,DisplayName,SchemaName,Description,PrimaryIdAttribute,PrimaryNameAttribute,EntitySetName,IsCustomEntity,IsManaged,ObjectTypeCode,MetadataId",
            logical_name
        );
        self.get_json(&endpoint).await
    }

    /// Get attributes for an entity
    pub async fn get_entity_attributes(&self, logical_name: &str) -> Result<Vec<AttributeMetadata>> {
        let endpoint = format!(
            "EntityDefinitions(LogicalName='{}')/Attributes?$select=LogicalName,DisplayName,SchemaName,AttributeType,AttributeTypeName,RequiredLevel,IsCustomAttribute,IsPrimaryId,IsPrimaryName,Description,MetadataId",
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
