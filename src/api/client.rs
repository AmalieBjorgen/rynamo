//! Dataverse Web API client

use crate::auth::AzureAuthenticator;
use anyhow::{Context, Result};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tokio::sync::RwLock;

/// HTTP client for Dataverse Web API
pub struct DataverseClient {
    http_client: Client,
    authenticator: Arc<AzureAuthenticator>,
    cached_token: RwLock<Option<String>>,
}

impl DataverseClient {
    /// Create a new Dataverse client
    pub fn new(authenticator: Arc<AzureAuthenticator>) -> Self {
        let http_client = Client::builder()
            .user_agent("Rynamo/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            authenticator,
            cached_token: RwLock::new(None),
        }
    }

    /// Get or refresh the authentication token
    async fn get_token(&self) -> Result<String> {
        // For simplicity, always get a fresh token
        // In production, we'd cache and check expiry
        let token = self.authenticator.get_token().await?;
        *self.cached_token.write().await = Some(token.clone());
        Ok(token)
    }

    /// Get the base API URL
    fn api_url(&self) -> String {
        format!("{}/api/data/v9.2", self.authenticator.environment_url())
    }

    /// Make an authenticated GET request
    pub async fn get(&self, endpoint: &str) -> Result<Response> {
        let token = self.get_token().await?;
        let url = if endpoint.starts_with("http") {
            endpoint.to_string()
        } else {
            format!("{}/{}", self.api_url(), endpoint.trim_start_matches('/'))
        };

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/json")
            .header("OData-MaxVersion", "4.0")
            .header("OData-Version", "4.0")
            .header("Prefer", "odata.include-annotations=\"*\"")
            .send()
            .await
            .context("Failed to send request to Dataverse")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, body);
        }

        Ok(response)
    }

    /// Make an authenticated GET request and deserialize JSON response
    pub async fn get_json<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let response = self.get(endpoint).await?;
        let data = response
            .json::<T>()
            .await
            .context("Failed to parse JSON response")?;
        Ok(data)
    }

    /// Get the environment URL
    pub fn environment_url(&self) -> &str {
        self.authenticator.environment_url()
    }

    /// Discover available Dataverse environments using the Global Discovery Service
    pub async fn discover_environments(&self) -> Result<Vec<crate::models::DiscoveryInstance>> {
        // Global Discovery Service endpoint
        let disco_url = "https://globaldisco.crm.dynamics.com/api/discovery/v2.0/Instances";
        // Scope for the discovery service
        let scope = "https://globaldisco.crm.dynamics.com/.default";

        let token = self.authenticator.get_token_for_scope(scope).await?;

        let response = self
            .http_client
            .get(disco_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to send request to Global Discovery Service")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Discovery request failed with status {}: {}", status, body);
        }

        let data: crate::models::DiscoveryResponse = response
            .json()
            .await
            .context("Failed to parse discovery response")?;

        Ok(data.value)
    }

    /// Execute a raw FetchXML query
    pub async fn execute_fetch_xml(&self, entity_set_name: &str, fetch_xml: &str) -> anyhow::Result<crate::models::QueryResult> {
        let endpoint = format!("{}?fetchXml={}", entity_set_name, urlencoding::encode(fetch_xml));
        let response = self.get(&endpoint).await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("FetchXML query failed with status {}: {}", status, body);
        }

        let body = response.text().await?;
        let json: serde_json::Value = serde_json::from_str(&body)
            .context("Failed to parse FetchXML response as JSON")?;
        
        Ok(crate::models::QueryResult::from_json(&json))
    }

    /// Get count of records where an attribute is not null
    pub async fn get_attribute_count(&self, entity_set_name: &str, attribute_name: &str) -> anyhow::Result<i64> {
        let endpoint = format!("{}/$count?$filter={} ne null", entity_set_name, attribute_name);
        let response = self.get(&endpoint).await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Count query failed with status {}: {}", status, body);
        }

        let body = response.text().await?;
        let count: i64 = body.parse().context("Failed to parse count response")?;
        Ok(count)
    }
}
