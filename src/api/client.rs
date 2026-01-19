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
}
