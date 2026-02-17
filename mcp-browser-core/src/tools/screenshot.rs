//! Take a screenshot (base64 PNG).

use crate::browser::BrowserManager;
use base64::Engine;
use chromiumoxide::cdp::browser_protocol::page::{
    CaptureScreenshotFormat, CaptureScreenshotParams,
};
use pmcp::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct ScreenshotInput {
    /// CSS selector to screenshot a specific element (optional, screenshots full viewport if omitted)
    #[schemars(description = "CSS selector to screenshot a specific element (optional)")]
    pub selector: Option<String>,

    /// Capture full scrollable page instead of just the viewport
    #[serde(default)]
    #[schemars(description = "Capture full scrollable page (default: false)")]
    pub full_page: bool,
}

pub async fn execute(
    manager: &Arc<BrowserManager>,
    input: ScreenshotInput,
) -> Result<serde_json::Value, Error> {
    input
        .validate()
        .map_err(|e| Error::validation(format!("Validation failed: {}", e)))?;

    let page = manager
        .page()
        .await
        .map_err(|e| Error::internal(format!("Browser error: {}", e)))?;

    let png_bytes = if let Some(ref selector) = input.selector {
        // Screenshot a specific element
        let element = page
            .find_element(selector)
            .await
            .map_err(|e| Error::internal(format!("Element not found '{}': {}", selector, e)))?;

        element
            .screenshot(CaptureScreenshotFormat::Png)
            .await
            .map_err(|e| Error::internal(format!("Screenshot failed: {}", e)))?
    } else {
        // Screenshot the page
        let params = CaptureScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .capture_beyond_viewport(input.full_page)
            .build();

        page.screenshot(params)
            .await
            .map_err(|e| Error::internal(format!("Screenshot failed: {}", e)))?
    };

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

    Ok(json!({
        "type": "image",
        "media_type": "image/png",
        "data": b64,
        "size_bytes": png_bytes.len()
    }))
}
