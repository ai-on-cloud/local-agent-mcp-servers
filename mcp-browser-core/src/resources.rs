//! MCP resources for browser state.
//!
//! Provides dynamic resources:
//! - `browser://page/dom` — current page's DOM as HTML
//! - `browser://page/url` — current page's URL
//!
//! These are registered as tools since PMCP's ResourceCollection currently
//! supports static resources. The dynamic functionality is available via
//! the `get_dom` and `get_url` tools.

use crate::browser::BrowserManager;
use pmcp::TypedTool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct GetDomInput {}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct GetUrlInput {}

/// Register resource-like tools onto the server builder.
pub fn register_resources(
    builder: pmcp::ServerBuilder,
    manager: Arc<BrowserManager>,
) -> pmcp::ServerBuilder {
    let m = manager.clone();
    let builder = builder.tool(
        "get_dom",
        TypedTool::new("get_dom", move |_input: GetDomInput, _extra| {
            let m = m.clone();
            Box::pin(async move {
                let page = m
                    .page()
                    .await
                    .map_err(|e| pmcp::Error::internal(format!("Browser error: {}", e)))?;

                let html = page
                    .content()
                    .await
                    .map_err(|e| pmcp::Error::internal(format!("Failed to get DOM: {}", e)))?;

                Ok(json!({
                    "dom": html,
                    "type": "text/html"
                }))
            })
        })
        .with_description("Get the current page's DOM as HTML."),
    );

    let m = manager;
    let builder = builder.tool(
        "get_url",
        TypedTool::new("get_url", move |_input: GetUrlInput, _extra| {
            let m = m.clone();
            Box::pin(async move {
                let page = m
                    .page()
                    .await
                    .map_err(|e| pmcp::Error::internal(format!("Browser error: {}", e)))?;

                let url = page
                    .url()
                    .await
                    .map_err(|e| pmcp::Error::internal(format!("Failed to get URL: {}", e)))?
                    .unwrap_or_default()
                    .to_string();

                Ok(json!({
                    "url": url
                }))
            })
        })
        .with_description("Get the current page's URL."),
    );

    builder
}
