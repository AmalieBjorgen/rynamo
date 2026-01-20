//! System Job API endpoints

use crate::api::DataverseClient;
use crate::models::SystemJob;
use anyhow::Result;

impl DataverseClient {
    /// Get system jobs (async operations)
    /// 
    /// # Arguments
    /// 
    /// * `top` - Max number of jobs to retrieve
    pub async fn get_system_jobs(&self, top: usize) -> Result<Vec<SystemJob>> {
        let select = "asyncoperationid,name,operationtype,statuscode,statecode,startedon,completedon,createdon,_createdby_value,message,friendlymessage,_regardingobjectid_value";
        let order_by = "createdon desc";
        let query = format!("asyncoperations?$select={}&$orderby={}&$top={}", select, order_by, top);

        let response: crate::models::odata::ODataResponse<SystemJob> = self.get_json(&query).await?;
        Ok(response.value)
    }

    /// Get system job details
    pub async fn get_system_job(&self, id: &str) -> Result<SystemJob> {
        let select = "asyncoperationid,name,operationtype,statuscode,statecode,startedon,completedon,createdon,_createdby_value,message,friendlymessage,_regardingobjectid_value";
        let query = format!("asyncoperations({})?$select={}", id, select);

        self.get_json(&query).await
    }
}
