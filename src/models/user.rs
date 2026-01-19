//! User, Team, and Security Role models

use serde::Deserialize;

/// System user metadata
#[derive(Debug, Clone, Deserialize)]
pub struct SystemUser {
    #[serde(rename = "systemuserid")]
    pub id: String,

    #[serde(rename = "fullname")]
    pub full_name: Option<String>,

    #[serde(rename = "domainname")]
    pub domain_name: Option<String>,

    #[serde(rename = "internalemailaddress")]
    pub email: Option<String>,

    #[serde(rename = "isdisabled")]
    pub is_disabled: Option<bool>,

    #[serde(rename = "businessunitid")]
    pub business_unit: Option<BusinessUnitRef>,

    #[serde(rename = "title")]
    pub title: Option<String>,

    #[serde(rename = "createdon")]
    pub created_on: Option<String>,
}

impl SystemUser {
    pub fn get_display_name(&self) -> String {
        self.full_name.clone().unwrap_or_else(|| 
            self.domain_name.clone().unwrap_or_else(|| "Unknown".to_string())
        )
    }

    pub fn get_status(&self) -> &str {
        if self.is_disabled.unwrap_or(false) {
            "Disabled"
        } else {
            "Enabled"
        }
    }
}

/// Business unit reference (expanded in queries)
#[derive(Debug, Clone, Deserialize)]
pub struct BusinessUnitRef {
    #[serde(rename = "businessunitid")]
    pub id: Option<String>,

    #[serde(rename = "name")]
    pub name: Option<String>,
}

/// Team metadata
#[derive(Debug, Clone, Deserialize)]
pub struct Team {
    #[serde(rename = "teamid")]
    pub id: String,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "teamtype")]
    pub team_type: Option<i32>,

    #[serde(rename = "description")]
    pub description: Option<String>,

    #[serde(rename = "isdefault")]
    pub is_default: Option<bool>,
}

impl Team {
    pub fn get_type_name(&self) -> &str {
        match self.team_type {
            Some(0) => "Owner",
            Some(1) => "Access",
            Some(2) => "AAD Security Group",
            Some(3) => "AAD Office Group",
            _ => "Unknown",
        }
    }
}

/// Security role metadata
#[derive(Debug, Clone, Deserialize)]
pub struct SecurityRole {
    #[serde(rename = "roleid")]
    pub id: String,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "businessunitid")]
    pub business_unit: Option<BusinessUnitRef>,

    #[serde(rename = "ismanaged")]
    pub is_managed: Option<bool>,
}

impl SecurityRole {
    pub fn get_business_unit_name(&self) -> String {
        self.business_unit
            .as_ref()
            .and_then(|bu| bu.name.clone())
            .unwrap_or_else(|| "-".to_string())
    }
}

/// Role assignment source (for display purposes)
#[derive(Debug, Clone)]
pub enum RoleSource {
    /// Directly assigned to the user
    Direct,
    /// Inherited from a team
    Team(String), // Team name
}

/// Combined role info with source
#[derive(Debug, Clone)]
pub struct RoleAssignment {
    pub role: SecurityRole,
    pub source: RoleSource,
}
