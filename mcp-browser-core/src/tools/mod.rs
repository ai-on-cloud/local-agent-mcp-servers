//! Tool registration for all browser automation tools.

pub mod click;
pub mod evaluate_script;
pub mod extract_table;
pub mod fill;
pub mod get_text;
pub mod handle_dialog;
pub mod hover;
pub mod list_pages;
pub mod navigate;
pub mod press_key;
pub mod screenshot;
pub mod select_page;
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
    // --- Navigation & page management ---

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
        "list_pages",
        TypedTool::new(
            "list_pages",
            move |input: list_pages::ListPagesInput, _extra| {
                let m = m.clone();
                Box::pin(async move { list_pages::execute(&m, input).await })
            },
        )
        .with_description("List all open browser pages (tabs) with their URLs and indices."),
    );

    let m = manager.clone();
    let builder = builder.tool(
        "select_page",
        TypedTool::new(
            "select_page",
            move |input: select_page::SelectPageInput, _extra| {
                let m = m.clone();
                Box::pin(async move { select_page::execute(&m, input).await })
            },
        )
        .with_description(
            "Switch the active browser page (tab) by index. Use list_pages to see available pages.",
        ),
    );

    let m = manager.clone();
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

    // --- Input automation ---

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
        .with_description(
            "Fill a form field identified by a CSS selector with the given text value.",
        ),
    );

    let m = manager.clone();
    let builder = builder.tool(
        "press_key",
        TypedTool::new(
            "press_key",
            move |input: press_key::PressKeyInput, _extra| {
                let m = m.clone();
                Box::pin(async move { press_key::execute(&m, input).await })
            },
        )
        .with_description(
            "Press a keyboard key, optionally with modifiers. Examples: 'Enter', 'Tab', 'Escape', 'Control+a', 'Shift+Tab'.",
        ),
    );

    let m = manager.clone();
    let builder = builder.tool(
        "hover",
        TypedTool::new("hover", move |input: hover::HoverInput, _extra| {
            let m = m.clone();
            Box::pin(async move { hover::execute(&m, input).await })
        })
        .with_description(
            "Hover over an element identified by a CSS selector. Triggers hover states, dropdowns, and tooltips.",
        ),
    );

    let m = manager.clone();
    let builder = builder.tool(
        "handle_dialog",
        TypedTool::new(
            "handle_dialog",
            move |input: handle_dialog::HandleDialogInput, _extra| {
                let m = m.clone();
                Box::pin(async move { handle_dialog::execute(&m, input).await })
            },
        )
        .with_description(
            "Accept or dismiss a JavaScript dialog (alert, confirm, prompt). Call this when a dialog is blocking the page.",
        ),
    );

    // --- Data extraction & debugging ---

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
        "evaluate_script",
        TypedTool::new(
            "evaluate_script",
            move |input: evaluate_script::EvaluateScriptInput, _extra| {
                let m = m.clone();
                Box::pin(async move { evaluate_script::execute(&m, input).await })
            },
        )
        .with_description(
            "Execute JavaScript in the browser page context. Returns the result of the expression.",
        ),
    );

    builder
}
