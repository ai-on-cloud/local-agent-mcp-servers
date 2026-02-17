//! Accept or dismiss a JavaScript dialog (alert, confirm, prompt).
//!
//! When a JS dialog (alert/confirm/prompt) appears, it blocks the page.
//! This tool sends the CDP `Page.handleJavaScriptDialog` command to
//! accept or dismiss the dialog, unblocking the page.

use crate::browser::BrowserManager;
use chromiumoxide::cdp::browser_protocol::page::HandleJavaScriptDialogParams;
use pmcp::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct HandleDialogInput {
    /// Whether to accept (true) or dismiss (false) the dialog
    #[schemars(description = "Whether to accept (true) or dismiss (false) the dialog")]
    pub accept: bool,

    /// Text to enter in a prompt dialog (only used for prompt dialogs)
    #[schemars(description = "Text to enter in a prompt() dialog (optional, only for prompt dialogs)")]
    pub prompt_text: Option<String>,
}

pub async fn execute(
    manager: &Arc<BrowserManager>,
    input: HandleDialogInput,
) -> Result<serde_json::Value, Error> {
    input
        .validate()
        .map_err(|e| Error::validation(format!("Validation failed: {}", e)))?;

    let page = manager
        .page()
        .await
        .map_err(|e| Error::internal(format!("Browser error: {}", e)))?;

    let mut params = HandleJavaScriptDialogParams::new(input.accept);
    if let Some(ref text) = input.prompt_text {
        params.prompt_text = Some(text.clone());
    }

    page.execute(params).await.map_err(|e| {
        Error::internal(format!(
            "Failed to handle dialog (is there an active dialog?): {}",
            e
        ))
    })?;

    let action = if input.accept { "accepted" } else { "dismissed" };

    Ok(json!({
        "status": action,
        "prompt_text": input.prompt_text
    }))
}
