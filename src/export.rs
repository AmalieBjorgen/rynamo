use crate::models::QueryResult;
use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Export formats
pub enum ExportFormat {
    Csv,
    Json,
}

/// Export query results to a file
pub fn export_results(results: &QueryResult, format: ExportFormat, path: &Path) -> Result<String> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    match format {
        ExportFormat::Csv => export_to_csv(results, path),
        ExportFormat::Json => export_to_json(results, path),
    }
}

fn export_to_csv(results: &QueryResult, path: &Path) -> Result<String> {
    let mut file = File::create(path)?;
    
    // Write header
    let header = results.columns.join(",");
    writeln!(file, "{}", header)?;

    // Write rows
    for row in &results.rows {
        let line = row.join(",");
        writeln!(file, "{}", line)?;
    }

    Ok(path.to_string_lossy().to_string())
}

fn export_to_json(results: &QueryResult, path: &Path) -> Result<String> {
    let mut file = File::create(path)?;
    
    // We want to export the raw data if available, or the tabular data
    // For now, let's export the tabular data as an array of objects
    let mut output = Vec::new();
    for row in &results.rows {
        let mut map = serde_json::Map::new();
        for (i, col) in results.columns.iter().enumerate() {
            map.insert(col.clone(), serde_json::Value::String(row[i].clone()));
        }
        output.push(serde_json::Value::Object(map));
    }

    let json = serde_json::to_string_pretty(&output)?;
    file.write_all(json.as_bytes())?;

    Ok(path.to_string_lossy().to_string())
}
