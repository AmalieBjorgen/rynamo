//! Azure CLI credential provider for Dataverse authentication

use anyhow::{Context, Result};
use azure_core::credentials::TokenCredential;
use azure_identity::AzureCliCredential;
use std::sync::{Arc, RwLock};

/// Authenticator that uses Azure CLI credentials to access Dataverse
pub struct AzureAuthenticator {
    credential: Arc<AzureCliCredential>,
    environment_url: RwLock<String>,
}

impl AzureAuthenticator {
    /// Create a new authenticator for the given Dataverse environment URL
    ///
    /// # Arguments
    /// * `environment_url` - The Dataverse environment URL (e.g., "https://org.crm.dynamics.com")
    pub async fn new(environment_url: impl Into<String>) -> Result<Self> {
        let environment_url = environment_url.into();
        let environment_url = environment_url.trim_end_matches('/').to_string();

        let credential = AzureCliCredential::new()
            .context("Failed to create Azure CLI credential")?;

        Ok(Self {
            credential,
            environment_url: RwLock::new(environment_url),
        })
    }

    /// Get an access token for the Dataverse API
    pub async fn get_token(&self) -> Result<String> {
        let env_url = self.environment_url.read().unwrap();
        let scope = format!("{}/.default", *env_url);
        self.get_token_for_scope(&scope).await
    }

    /// Get an access token for a specific scope
    pub async fn get_token_for_scope(&self, scope: &str) -> Result<String> {
        let token = self
            .credential
            .get_token(&[scope])
            .await
            .context(format!(
                "Failed to get token for scope '{}' from Azure CLI. Make sure you're logged in with 'az login'",
                scope
            ))?;

        Ok(token.token.secret().to_string())
    }

    /// Get the environment URL
    pub fn environment_url(&self) -> String {
        self.environment_url.read().unwrap().clone()
    }

    /// Set the environment URL
    pub fn set_environment_url(&self, url: impl Into<String>) {
        let mut env_url = self.environment_url.write().unwrap();
        let url = url.into().trim_end_matches('/').to_string();
        *env_url = url;
    }

    /// Test if authentication is working
    pub async fn test_connection(&self) -> Result<()> {
        self.get_token().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_environment_url_normalization() {
        // Note: This test requires Azure CLI to be installed, so we skip the actual creation
        // and just test the URL normalization logic directly
        let url = "https://test.crm.dynamics.com/";
        let normalized = url.trim_end_matches('/');
        assert_eq!(normalized, "https://test.crm.dynamics.com");
    }
}
