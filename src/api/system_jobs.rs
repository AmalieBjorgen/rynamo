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
    /// * `filter` - Optional OData filter string
    pub async fn get_system_jobs(&self, top: usize, filter: Option<&str>) -> Result<(Vec<SystemJob>, Option<String>)> {
        let select = "asyncoperationid,name,operationtype,statuscode,statecode,startedon,completedon,createdon,_createdby_value,message,friendlymessage,_regardingobjectid_value";
        let order_by = "createdon desc";
        
        let mut query = format!("asyncoperations?$select={}&$orderby={}&$top={}", select, order_by, top);
        
        if let Some(f) = filter {
            if !f.is_empty() {
                query.push_str(&format!("&$filter={}", f));
            }
        }

        let response: crate::models::odata::ODataResponse<SystemJob> = self.get_json(&query).await?;
        Ok((response.value, response.next_link))
    }

    /// Get next page of system jobs
    pub async fn get_next_page_system_jobs(&self, next_link: &str) -> Result<(Vec<SystemJob>, Option<String>)> {
        let response: crate::models::odata::ODataResponse<SystemJob> = self.get_json(next_link).await?;
        Ok((response.value, response.next_link))
    }

    /// Get system job details
    pub async fn get_system_job(&self, id: &str) -> Result<SystemJob> {
        let select = "asyncoperationid,name,operationtype,statuscode,statecode,startedon,completedon,createdon,_createdby_value,message,friendlymessage,_regardingobjectid_value";
        let query = format!("asyncoperations({})?$select={}", id, select);

        self.get_json(&query).await
    }
}
