//! Solution models

use serde::Deserialize;

/// Solution metadata
#[derive(Debug, Clone, Deserialize)]
pub struct Solution {
    #[serde(rename = "solutionid")]
    pub solution_id: String,

    #[serde(rename = "uniquename")]
    pub unique_name: String,

    #[serde(rename = "friendlyname")]
    pub friendly_name: Option<String>,

    #[serde(rename = "version")]
    pub version: Option<String>,

    #[serde(rename = "ismanaged")]
    pub is_managed: Option<bool>,

    #[serde(rename = "description")]
    pub description: Option<String>,

    #[serde(rename = "installedon")]
    pub installed_on: Option<String>,
}

impl Solution {
    /// Get the display name
    pub fn get_display_name(&self) -> String {
        self.friendly_name
            .clone()
            .unwrap_or_else(|| self.unique_name.clone())
    }
}

/// Solution component
#[derive(Debug, Clone, Deserialize)]
pub struct SolutionComponent {
    #[serde(rename = "solutioncomponentid")]
    pub solution_component_id: String,

    #[serde(rename = "componenttype")]
    pub component_type: Option<i32>,

    #[serde(rename = "objectid")]
    pub object_id: Option<String>,

    #[serde(rename = "rootcomponentbehavior")]
    pub root_component_behavior: Option<i32>,
}

impl SolutionComponent {
    /// Get the component type as an enum
    pub fn get_component_type(&self) -> ComponentType {
        ComponentType::from_code(self.component_type.unwrap_or(0))
    }
}

/// Component types in solutions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentType {
    Entity,
    Attribute,
    Relationship,
    OptionSet,
    EntityKey,
    Role,
    RolePrivilege,
    FieldSecurityProfile,
    FieldPermission,
    PluginAssembly,
    PluginType,
    SdkMessageProcessingStep,
    Workflow,
    WebResource,
    SiteMap,
    ConnectionRole,
    Report,
    SystemForm,
    SavedQuery,
    SavedQueryVisualization,
    RibbonCustomization,
    EntityRibbonSetting,
    EmailTemplate,
    ContractTemplate,
    KbArticleTemplate,
    MailMergeTemplate,
    DuplicateRule,
    RibbonCommand,
    RibbonContextGroup,
    RibbonDiff,
    ManagedProperty,
    EntityRelationship,
    Custom,
    ProcessTrigger,
    AppModule,
    Unknown(i32),
}

impl ComponentType {
    /// Convert from Dataverse component type code
    pub fn from_code(code: i32) -> Self {
        match code {
            1 => Self::Entity,
            2 => Self::Attribute,
            3 => Self::Relationship,
            9 => Self::OptionSet,
            14 => Self::EntityKey,
            20 => Self::Role,
            21 => Self::RolePrivilege,
            70 => Self::FieldSecurityProfile,
            71 => Self::FieldPermission,
            91 => Self::PluginAssembly,
            90 => Self::PluginType,
            92 => Self::SdkMessageProcessingStep,
            29 => Self::Workflow,
            61 => Self::WebResource,
            62 => Self::SiteMap,
            63 => Self::ConnectionRole,
            31 => Self::Report,
            60 => Self::SystemForm,
            26 => Self::SavedQuery,
            59 => Self::SavedQueryVisualization,
            50 => Self::RibbonCustomization,
            48 => Self::EntityRibbonSetting,
            36 => Self::EmailTemplate,
            37 => Self::ContractTemplate,
            38 => Self::KbArticleTemplate,
            39 => Self::MailMergeTemplate,
            44 => Self::DuplicateRule,
            52 => Self::RibbonCommand,
            53 => Self::RibbonContextGroup,
            55 => Self::RibbonDiff,
            13 => Self::ManagedProperty,
            10 => Self::EntityRelationship,
            66 => Self::Custom,
            78 => Self::ProcessTrigger,
            80 => Self::AppModule,
            _ => Self::Unknown(code),
        }
    }

    /// Get a display name for the component type
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Entity => "Entity",
            Self::Attribute => "Attribute",
            Self::Relationship => "Relationship",
            Self::OptionSet => "Option Set",
            Self::EntityKey => "Entity Key",
            Self::Role => "Security Role",
            Self::RolePrivilege => "Role Privilege",
            Self::FieldSecurityProfile => "Field Security Profile",
            Self::FieldPermission => "Field Permission",
            Self::PluginAssembly => "Plugin Assembly",
            Self::PluginType => "Plugin Type",
            Self::SdkMessageProcessingStep => "Plugin Step",
            Self::Workflow => "Workflow/Flow",
            Self::WebResource => "Web Resource",
            Self::SiteMap => "Site Map",
            Self::ConnectionRole => "Connection Role",
            Self::Report => "Report",
            Self::SystemForm => "Form",
            Self::SavedQuery => "View",
            Self::SavedQueryVisualization => "Chart",
            Self::RibbonCustomization => "Ribbon",
            Self::EntityRibbonSetting => "Entity Ribbon",
            Self::EmailTemplate => "Email Template",
            Self::ContractTemplate => "Contract Template",
            Self::KbArticleTemplate => "KB Article Template",
            Self::MailMergeTemplate => "Mail Merge Template",
            Self::DuplicateRule => "Duplicate Rule",
            Self::RibbonCommand => "Ribbon Command",
            Self::RibbonContextGroup => "Ribbon Context",
            Self::RibbonDiff => "Ribbon Diff",
            Self::ManagedProperty => "Managed Property",
            Self::EntityRelationship => "Entity Relationship",
            Self::Custom => "Custom",
            Self::ProcessTrigger => "Process Trigger",
            Self::AppModule => "App Module",
            Self::Unknown(_) => "Unknown",
        }
    }
}
