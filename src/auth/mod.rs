//! Auth module for Azure CLI authentication
//!
//! Provides token acquisition using Azure CLI credentials for Dataverse API access.

mod azure_cli;

pub use azure_cli::AzureAuthenticator;
