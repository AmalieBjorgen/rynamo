//! Solution metadata API

use super::DataverseClient;
use crate::models::{Solution, SolutionComponent, SolutionComponentLayer};
use crate::models::odata::ODataResponse;
use anyhow::Result;

impl DataverseClient {
    /// Get layers for a specific component
    pub async fn get_solution_layers(&self, component_id: &str, component_type: i32) -> Result<Vec<SolutionComponentLayer>> {
        let endpoint = format!(
            "msdyn_solutioncomponentlayers?$filter=msdyn_componentid eq '{}' and msdyn_componenttype eq {}",
            component_id, component_type
        );
        let response: ODataResponse<SolutionComponentLayer> = self.get_json(&endpoint).await?;
        Ok(response.value)
    }
    /// Get all solutions
    pub async fn get_solutions(&self) -> Result<Vec<Solution>> {
        let response: ODataResponse<Solution> = self
            .get_json("solutions?$select=solutionid,uniquename,friendlyname,version,ismanaged,publisherid,description,installedon&$orderby=friendlyname")
            .await?;
        Ok(response.value)
    }

    /// Get components in a solution
    pub async fn get_solution_components(&self, solution_id: &str) -> Result<Vec<SolutionComponent>> {
        let endpoint = format!(
            "solutioncomponents?$filter=_solutionid_value eq {}&$select=componenttype,objectid,solutioncomponentid,rootcomponentbehavior",
            solution_id
        );
        let response: ODataResponse<SolutionComponent> = self.get_json(&endpoint).await?;
        Ok(response.value)
    }
}
