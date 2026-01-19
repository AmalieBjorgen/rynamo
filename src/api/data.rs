//! Data query API

use super::DataverseClient;
use anyhow::Result;
use serde_json::Value as JsonValue;

impl DataverseClient {
    /// Execute a raw OData query and return JSON
    pub async fn execute_query(&self, query_url: &str) -> Result<JsonValue> {
        let response = self.get(query_url).await?;
        let json: JsonValue = response.json().await?;
        Ok(json)
    }
}
