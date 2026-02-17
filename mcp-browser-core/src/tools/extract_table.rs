//! Extract an HTML table as JSON.

use crate::browser::BrowserManager;
use pmcp::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct ExtractTableInput {
    /// CSS selector of the table element
    #[validate(length(min = 1))]
    #[schemars(description = "CSS selector of the <table> element to extract")]
    pub selector: String,
}

/// JavaScript that extracts a table into an array of row objects.
/// Uses the first row (th or td) as column headers.
const EXTRACT_TABLE_JS: &str = r#"
(selector) => {
    const table = document.querySelector(selector);
    if (!table) return JSON.stringify({ error: "Table not found" });

    const rows = Array.from(table.querySelectorAll('tr'));
    if (rows.length === 0) return JSON.stringify({ rows: [], headers: [] });

    // Extract headers from first row
    const headerRow = rows[0];
    const headers = Array.from(headerRow.querySelectorAll('th, td'))
        .map(cell => cell.textContent.trim());

    // Extract data rows
    const dataRows = rows.slice(1).map(row => {
        const cells = Array.from(row.querySelectorAll('td, th'));
        const obj = {};
        cells.forEach((cell, i) => {
            const key = i < headers.length ? headers[i] : `column_${i}`;
            obj[key] = cell.textContent.trim();
        });
        return obj;
    });

    return JSON.stringify({ headers: headers, rows: dataRows });
}
"#;

pub async fn execute(
    manager: &Arc<BrowserManager>,
    input: ExtractTableInput,
) -> Result<serde_json::Value, Error> {
    input
        .validate()
        .map_err(|e| Error::validation(format!("Validation failed: {}", e)))?;

    let page = manager
        .page()
        .await
        .map_err(|e| Error::internal(format!("Browser error: {}", e)))?;

    let js = format!(
        "({})({})",
        EXTRACT_TABLE_JS,
        serde_json::to_string(&input.selector).unwrap()
    );

    let result: String = page
        .evaluate_expression(js)
        .await
        .map_err(|e| Error::internal(format!("Table extraction failed: {}", e)))?
        .into_value()
        .map_err(|e| Error::internal(format!("Failed to parse JS result: {:?}", e)))?;

    let parsed: serde_json::Value = serde_json::from_str(&result)
        .map_err(|e| Error::internal(format!("Failed to parse table JSON: {}", e)))?;

    Ok(parsed)
}
