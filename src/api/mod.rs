//! API module for Dataverse Web API interactions

mod client;
pub mod metadata;
pub mod solutions;
pub mod users;

pub use client::DataverseClient;
