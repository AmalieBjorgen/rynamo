//! User, Team, and Security Role API endpoints

use super::DataverseClient;
use crate::models::{SecurityRole, SystemUser, Team};
use crate::models::odata::ODataResponse;
use anyhow::Result;

impl DataverseClient {
    /// Get all enabled system users
    pub async fn get_users(&self) -> Result<Vec<SystemUser>> {
        let response: ODataResponse<SystemUser> = self
            .get_json("systemusers?$select=systemuserid,fullname,domainname,internalemailaddress,isdisabled,title,createdon&$expand=businessunitid($select=businessunitid,name)&$filter=isdisabled eq false&$orderby=fullname")
            .await?;
        Ok(response.value)
    }

    /// Get all system users including disabled
    pub async fn get_all_users(&self) -> Result<Vec<SystemUser>> {
        let response: ODataResponse<SystemUser> = self
            .get_json("systemusers?$select=systemuserid,fullname,domainname,internalemailaddress,isdisabled,title,createdon&$expand=businessunitid($select=businessunitid,name)&$orderby=fullname")
            .await?;
        Ok(response.value)
    }

    /// Get teams that a user belongs to
    pub async fn get_user_teams(&self, user_id: &str) -> Result<Vec<Team>> {
        let endpoint = format!(
            "systemusers({})/teammembership_association?$select=teamid,name,teamtype,description,isdefault",
            user_id
        );
        let response: ODataResponse<Team> = self.get_json(&endpoint).await?;
        Ok(response.value)
    }

    /// Get security roles directly assigned to a user
    pub async fn get_user_roles(&self, user_id: &str) -> Result<Vec<SecurityRole>> {
        let endpoint = format!(
            "systemusers({})/systemuserroles_association?$select=roleid,name,ismanaged&$expand=businessunitid($select=businessunitid,name)",
            user_id
        );
        let response: ODataResponse<SecurityRole> = self.get_json(&endpoint).await?;
        Ok(response.value)
    }

    /// Get security roles assigned to a team
    pub async fn get_team_roles(&self, team_id: &str) -> Result<Vec<SecurityRole>> {
        let endpoint = format!(
            "teams({})/teamroles_association?$select=roleid,name,ismanaged&$expand=businessunitid($select=businessunitid,name)",
            team_id
        );
        let response: ODataResponse<SecurityRole> = self.get_json(&endpoint).await?;
        Ok(response.value)
    }
}
