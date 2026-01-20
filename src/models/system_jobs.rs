//! System Job (AsyncOperation) models

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SystemJob {
    #[serde(rename = "asyncoperationid")]
    pub id: String,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "operationtype")]
    pub operation_type: Option<i32>,

    #[serde(rename = "statuscode")]
    pub status_code: Option<i32>,

    #[serde(rename = "statecode")]
    pub state_code: Option<i32>,

    #[serde(rename = "startedon")]
    pub started_on: Option<String>,

    #[serde(rename = "completedon")]
    pub completed_on: Option<String>,

    #[serde(rename = "createdon")]
    pub created_on: Option<String>,

    #[serde(rename = "_createdby_value")]
    pub created_by: Option<String>,
    
    #[serde(rename = "_createdby_value@OData.Community.Display.V1.FormattedValue")]
    pub created_by_name: Option<String>,

    #[serde(rename = "message")]
    pub message: Option<String>,
    
    #[serde(rename = "friendlymessage")]
    pub friendly_message: Option<String>,

    #[serde(rename = "_regardingobjectid_value")]
    pub regarding_object_id: Option<String>,
    
    #[serde(rename = "_regardingobjectid_value@OData.Community.Display.V1.FormattedValue")]
    pub regarding_object_name: Option<String>,
    
    #[serde(rename = "_regardingobjectid_value@Microsoft.Dynamics.CRM.lookuplogicalname")]
    pub regarding_object_type: Option<String>,
}

impl SystemJob {
    pub fn get_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| "Unknown Job".to_string())
    }

    pub fn get_status_label(&self) -> String {
        match self.status_code {
            Some(0) => "Waiting For Resources",
            Some(10) => "Waiting",
            Some(20) => "In Progress",
            Some(21) => "Pausing",
            Some(22) => "Canceling",
            Some(30) => "Succeeded",
            Some(31) => "Failed",
            Some(32) => "Canceled",
            _ => "Unknown",
        }.to_string()
    }
    
    pub fn get_state_label(&self) -> String {
        match self.state_code {
            Some(0) => "Ready",
            Some(1) => "Suspended",
            Some(2) => "Locked",
            Some(3) => "Completed",
            _ => "Unknown",
        }.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_job_deserialization() {
        let json = r#"
        {
            "asyncoperationid": "1234-5678",
            "name": "Test Job",
            "operationtype": 10,
            "statuscode": 30,
            "statecode": 3,
            "startedon": "2023-01-01T12:00:00Z",
            "completedon": "2023-01-01T12:05:00Z",
            "createdon": "2023-01-01T11:55:00Z",
            "_createdby_value": "user-guid",
            "message": "Job succeeded",
            "friendlymessage": "All good",
            "_regardingobjectid_value": "reg-guid",
            "_regardingobjectid_value@OData.Community.Display.V1.FormattedValue": "Regarding Entity",
            "_regardingobjectid_value@Microsoft.Dynamics.CRM.lookuplogicalname": "account"
        }
        "#;

        let job: SystemJob = serde_json::from_str(json).unwrap();
        
        assert_eq!(job.id, "1234-5678");
        assert_eq!(job.name.as_deref(), Some("Test Job"));
        assert_eq!(job.status_code, Some(30));
        assert_eq!(job.get_status_label(), "Succeeded");
        assert_eq!(job.get_state_label(), "Completed");
    }
}
