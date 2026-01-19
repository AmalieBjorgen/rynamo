//! Solution metadata API

use super::DataverseClient;
use crate::models::{ODataResponse, Solution, SolutionComponent};
use anyhow::Result;

impl DataverseClient {
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
