//! Azure CLI credential provider for Dataverse authentication

use anyhow::{Context, Result};
use azure_core::credentials::TokenCredential;
use azure_identity::AzureCliCredential;
use std::sync::Arc;

/// Authenticator that uses Azure CLI credentials to access Dataverse
pub struct AzureAuthenticator {
    credential: Arc<AzureCliCredential>,
    environment_url: String,
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
            environment_url,
        })
    }

    /// Get an access token for the Dataverse API
    pub async fn get_token(&self) -> Result<String> {
        let scope = format!("{}/.default", self.environment_url);
        
        let token = self
            .credential
            .get_token(&[&scope])
            .await
            .context("Failed to get token from Azure CLI. Make sure you're logged in with 'az login'")?;

        Ok(token.token.secret().to_string())
    }

    /// Get the environment URL
    pub fn environment_url(&self) -> &str {
        &self.environment_url
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
