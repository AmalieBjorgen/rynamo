//! Query builder models and structures

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// A query definition for executing against an entity
#[derive(Debug, Clone, Default)]
pub struct QueryDefinition {
    /// Entity logical name to query
    pub entity_name: String,
    /// Entity set name (for URL)
    pub entity_set_name: String,
    /// Columns to select (empty = all)
    pub select: Vec<String>,
    /// Filter expression
    pub filter: String,
    /// Order by columns
    pub order_by: String,
    /// Maximum records to return
    pub top: Option<usize>,
    /// Skip records (for pagination)
    pub skip: Option<usize>,
}

impl QueryDefinition {
    /// Build the OData query URL path
    pub fn build_url(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        if !self.select.is_empty() {
            parts.push(format!("$select={}", self.select.join(",")));
        }

        if !self.filter.is_empty() {
            parts.push(format!("$filter={}", self.filter));
        }

        if !self.order_by.is_empty() {
            parts.push(format!("$orderby={}", self.order_by));
        }

        if let Some(top) = self.top {
            parts.push(format!("$top={}", top));
        }

        if let Some(skip) = self.skip {
            parts.push(format!("$skip={}", skip));
        }

        if parts.is_empty() {
            self.entity_set_name.clone()
        } else {
            format!("{}?{}", self.entity_set_name, parts.join("&"))
        }
    }

    /// Clear the query
    pub fn clear(&mut self) {
        self.select.clear();
        self.filter.clear();
        self.order_by.clear();
        self.top = None;
        self.skip = None;
    }
}

/// Query result from executing a query
#[derive(Debug, Clone, Default)]
pub struct QueryResult {
    /// Column headers
    pub columns: Vec<String>,
    /// Rows of data (each row is a vec of string values)
    pub rows: Vec<Vec<String>>,
    /// Total count if available
    pub count: Option<usize>,
    /// Error message if query failed
    pub error: Option<String>,
    /// Raw JSON response for inspection
    pub raw_json: Option<String>,
}

impl QueryResult {
    /// Create from JSON response
    pub fn from_json(json: &JsonValue) -> Self {
        let mut result = QueryResult::default();

        // Get the value array
        let records = match json.get("value") {
            Some(JsonValue::Array(arr)) => arr,
            _ => {
                result.error = Some("Invalid response format: missing 'value' array".to_string());
                return result;
            }
        };

        if records.is_empty() {
            return result;
        }

        // Extract columns from first record
        if let Some(first) = records.first() {
            if let JsonValue::Object(obj) = first {
                result.columns = obj
                    .keys()
                    .filter(|k| !k.starts_with('@')) // Skip OData annotations
                    .cloned()
                    .collect();
                result.columns.sort();
            }
        }

        // Extract rows
        for record in records {
            if let JsonValue::Object(obj) = record {
                let row: Vec<String> = result
                    .columns
                    .iter()
                    .map(|col| {
                        obj.get(col)
                            .map(|v| format_json_value(v))
                            .unwrap_or_else(|| "-".to_string())
                    })
                    .collect();
                result.rows.push(row);
            }
        }

        // Get count if available
        if let Some(JsonValue::Number(n)) = json.get("@odata.count") {
            result.count = n.as_u64().map(|n| n as usize);
        }

        result
    }
}

/// Format a JSON value for display
fn format_json_value(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => "-".to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::String(s) => s.clone(),
        JsonValue::Array(arr) => format!("[{} items]", arr.len()),
        JsonValue::Object(_) => "{...}".to_string(),
    }
}

/// Query input field being edited
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QueryField {
    #[default]
    Entity,
    Select,
    Filter,
    OrderBy,
    Top,
}

impl QueryField {
    pub fn next(&self) -> Self {
        match self {
            Self::Entity => Self::Select,
            Self::Select => Self::Filter,
            Self::Filter => Self::OrderBy,
            Self::OrderBy => Self::Top,
            Self::Top => Self::Entity,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Entity => Self::Top,
            Self::Select => Self::Entity,
            Self::Filter => Self::Select,
            Self::OrderBy => Self::Filter,
            Self::Top => Self::OrderBy,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Entity => "Entity",
            Self::Select => "$select",
            Self::Filter => "$filter",
            Self::OrderBy => "$orderby",
            Self::Top => "$top",
        }
    }
}
