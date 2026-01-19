//! OData response wrapper

use serde::Deserialize;

/// Generic OData response with value array
#[derive(Debug, Deserialize)]
pub struct ODataResponse<T> {
    #[serde(rename = "value")]
    pub value: Vec<T>,

    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,

    #[serde(rename = "@odata.count")]
    pub count: Option<i64>,
}
