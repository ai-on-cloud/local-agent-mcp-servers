//! Tool registration for all browser automation tools.

pub mod click;
pub mod extract_table;
pub mod fill;
pub mod get_text;
pub mod navigate;
pub mod screenshot;
pub mod wait;

use crate::browser::BrowserManager;
use pmcp::TypedTool;
use std::sync::Arc;

/// Register all browser tools onto the server builder.
///
/// Each tool captures an `Arc<BrowserManager>` for browser access.
pub fn register_tools(
    builder: pmcp::ServerBuilder,
    manager: Arc<BrowserManager>,
) -> pmcp::ServerBuilder {
    let m = manager.clone();
    let builder = builder.tool(
        "navigate",
        TypedTool::new("navigate", move |input: navigate::NavigateInput, _extra| {
            let m = m.clone();
            Box::pin(async move { navigate::execute(&m, input).await })
        })
        .with_description("Navigate to a URL. Returns the final URL after any redirects."),
    );

    let m = manager.clone();
    let builder = builder.tool(
        "click",
        TypedTool::new("click", move |input: click::ClickInput, _extra| {
            let m = m.clone();
            Box::pin(async move { click::execute(&m, input).await })
        })
        .with_description("Click an element identified by a CSS selector."),
    );

    let m = manager.clone();
    let builder = builder.tool(
        "fill",
        TypedTool::new("fill", move |input: fill::FillInput, _extra| {
            let m = m.clone();
            Box::pin(async move { fill::execute(&m, input).await })
        })
        .with_description("Fill a form field identified by a CSS selector with the given text value."),
    );

    let m = manager.clone();
    let builder = builder.tool(
        "screenshot",
        TypedTool::new(
            "screenshot",
            move |input: screenshot::ScreenshotInput, _extra| {
                let m = m.clone();
                Box::pin(async move { screenshot::execute(&m, input).await })
            },
        )
        .with_description(
            "Take a screenshot of the page or a specific element. Returns base64-encoded PNG.",
        ),
    );

    let m = manager.clone();
    let builder = builder.tool(
        "extract_table",
        TypedTool::new(
            "extract_table",
            move |input: extract_table::ExtractTableInput, _extra| {
                let m = m.clone();
                Box::pin(async move { extract_table::execute(&m, input).await })
            },
        )
        .with_description(
            "Extract an HTML table as JSON. Returns headers and rows as structured data.",
        ),
    );

    let m = manager.clone();
    let builder = builder.tool(
        "get_text",
        TypedTool::new(
            "get_text",
            move |input: get_text::GetTextInput, _extra| {
                let m = m.clone();
                Box::pin(async move { get_text::execute(&m, input).await })
            },
        )
        .with_description("Get the text content of an element identified by a CSS selector."),
    );

    let m = manager;
    let builder = builder.tool(
        "wait",
        TypedTool::new("wait", move |input: wait::WaitInput, _extra| {
            let m = m.clone();
            Box::pin(async move { wait::execute(&m, input).await })
        })
        .with_description(
            "Wait for a CSS selector to appear on the page, or wait for a specified duration.",
        ),
    );

    builder
}
