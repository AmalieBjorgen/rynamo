//! Entity and attribute metadata models

use serde::Deserialize;

/// Localized label structure from Dataverse
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LocalizedLabel {
    #[serde(rename = "LocalizedLabels")]
    pub localized_labels: Option<Vec<LabelValue>>,
    #[serde(rename = "UserLocalizedLabel")]
    pub user_localized_label: Option<LabelValue>,
}

impl LocalizedLabel {
    /// Get the user's localized label text, or empty string if none
    pub fn get_label(&self) -> String {
        self.user_localized_label
            .as_ref()
            .and_then(|l| l.label.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LabelValue {
    #[serde(rename = "Label")]
    pub label: Option<String>,
    #[serde(rename = "LanguageCode")]
    pub language_code: Option<i32>,
}

/// Entity metadata from EntityDefinitions
#[derive(Debug, Clone, Deserialize)]
pub struct EntityMetadata {
    #[serde(rename = "MetadataId")]
    pub metadata_id: String,

    #[serde(rename = "LogicalName")]
    pub logical_name: String,

    #[serde(rename = "SchemaName")]
    pub schema_name: Option<String>,

    #[serde(rename = "DisplayName")]
    pub display_name: Option<LocalizedLabel>,

    #[serde(rename = "Description")]
    pub description: Option<LocalizedLabel>,

    #[serde(rename = "PrimaryIdAttribute")]
    pub primary_id_attribute: Option<String>,

    #[serde(rename = "PrimaryNameAttribute")]
    pub primary_name_attribute: Option<String>,

    #[serde(rename = "EntitySetName")]
    pub entity_set_name: Option<String>,

    #[serde(rename = "IsCustomEntity")]
    pub is_custom_entity: Option<bool>,

    #[serde(rename = "IsManaged")]
    pub is_managed: Option<bool>,

    #[serde(rename = "ObjectTypeCode")]
    pub object_type_code: Option<i32>,
}

impl EntityMetadata {
    /// Get the display name or fall back to logical name
    pub fn get_display_name(&self) -> String {
        self.display_name
            .as_ref()
            .map(|d| d.get_label())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.logical_name.clone())
    }

    /// Get description text
    pub fn get_description(&self) -> String {
        self.description
            .as_ref()
            .map(|d| d.get_label())
            .unwrap_or_default()
    }
}

/// Attribute metadata
#[derive(Debug, Clone, Deserialize)]
pub struct AttributeMetadata {
    #[serde(rename = "MetadataId")]
    pub metadata_id: String,

    #[serde(rename = "LogicalName")]
    pub logical_name: String,

    #[serde(rename = "SchemaName")]
    pub schema_name: Option<String>,

    #[serde(rename = "DisplayName")]
    pub display_name: Option<LocalizedLabel>,

    #[serde(rename = "Description")]
    pub description: Option<LocalizedLabel>,

    #[serde(rename = "AttributeType")]
    pub attribute_type: Option<String>,

    #[serde(rename = "AttributeTypeName")]
    pub attribute_type_name: Option<AttributeTypeName>,

    #[serde(rename = "RequiredLevel")]
    pub required_level: Option<RequiredLevel>,

    #[serde(rename = "IsCustomAttribute")]
    pub is_custom_attribute: Option<bool>,

    #[serde(rename = "IsPrimaryId")]
    pub is_primary_id: Option<bool>,

    #[serde(rename = "IsPrimaryName")]
    pub is_primary_name: Option<bool>,

    #[serde(rename = "MaxLength")]
    pub max_length: Option<i32>,

    #[serde(rename = "MinValue")]
    pub min_value: Option<f64>,

    #[serde(rename = "MaxValue")]
    pub max_value: Option<f64>,
}

impl AttributeMetadata {
    /// Get the display name or fall back to logical name
    pub fn get_display_name(&self) -> String {
        self.display_name
            .as_ref()
            .map(|d| d.get_label())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.logical_name.clone())
    }

    /// Get the attribute type as a string
    pub fn get_type_name(&self) -> String {
        self.attribute_type_name
            .as_ref()
            .map(|t| t.value.clone())
            .or_else(|| self.attribute_type.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    /// Check if the attribute is required
    pub fn is_required(&self) -> bool {
        self.required_level
            .as_ref()
            .map(|r| r.value == "ApplicationRequired" || r.value == "SystemRequired")
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AttributeTypeName {
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequiredLevel {
    #[serde(rename = "Value")]
    pub value: String,
}

/// Relationship metadata
#[derive(Debug, Clone, Deserialize)]
pub struct RelationshipMetadata {
    #[serde(rename = "SchemaName")]
    pub schema_name: Option<String>,

    // For 1:N and N:1
    #[serde(rename = "ReferencingEntity")]
    pub referencing_entity: Option<String>,

    #[serde(rename = "ReferencingAttribute")]
    pub referencing_attribute: Option<String>,

    #[serde(rename = "ReferencedEntity")]
    pub referenced_entity: Option<String>,

    #[serde(rename = "ReferencedAttribute")]
    pub referenced_attribute: Option<String>,

    // For N:N
    #[serde(rename = "Entity1LogicalName")]
    pub entity1_logical_name: Option<String>,

    #[serde(rename = "Entity2LogicalName")]
    pub entity2_logical_name: Option<String>,

    #[serde(rename = "IntersectEntityName")]
    pub intersect_entity_name: Option<String>,
}

impl RelationshipMetadata {
    /// Get a descriptive name for the relationship
    pub fn get_name(&self) -> String {
        self.schema_name.clone().unwrap_or_else(|| "Unknown".to_string())
    }

    /// Get the related entity name (for navigation)
    pub fn get_related_entity(&self, from_entity: &str) -> Option<String> {
        // For 1:N/N:1, return the other entity
        if let Some(ref_entity) = &self.referenced_entity {
            if ref_entity != from_entity {
                return Some(ref_entity.clone());
            }
        }
        if let Some(refing_entity) = &self.referencing_entity {
            if refing_entity != from_entity {
                return Some(refing_entity.clone());
            }
        }
        // For N:N
        if let Some(e1) = &self.entity1_logical_name {
            if e1 != from_entity {
                return Some(e1.clone());
            }
        }
        if let Some(e2) = &self.entity2_logical_name {
            if e2 != from_entity {
                return Some(e2.clone());
            }
        }
        None
    }
}
