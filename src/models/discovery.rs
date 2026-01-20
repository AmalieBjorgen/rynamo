use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DiscoveryInstance {
    #[serde(rename = "Id")]
    pub id: String,
    
    #[serde(rename = "Url")]
    pub url: String,
    
    #[serde(rename = "UniqueName")]
    pub unique_name: String,
    
    #[serde(rename = "FriendlyName")]
    pub friendly_name: String,
    
    #[serde(rename = "Region")]
    pub region: String,
    
    #[serde(rename = "Version")]
    pub version: String,
    
    #[serde(rename = "State")]
    pub state: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscoveryResponse {
    pub value: Vec<DiscoveryInstance>,
}
