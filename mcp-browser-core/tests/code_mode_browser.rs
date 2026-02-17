//! Integration tests for code mode with a real browser.
//!
//! These tests launch a headless Chrome/Chromium instance via CDP and execute
//! code mode scripts against real websites. They are `#[ignore]` by default
//! because they require a Chrome/Chromium binary installed.
//!
//! Run with:
//!   cargo test -p mcp-browser-core --test code_mode_browser -- --ignored

use mcp_browser_core::browser::{BrowserManager, BrowserManagerConfig};
use mcp_browser_core::code_mode;
use mcp_browser_core::profile::ProfileManager;
use std::sync::Arc;

/// Create a headless BrowserManager for testing.
fn test_manager() -> Arc<BrowserManager> {
    let profile_manager = Arc::new(ProfileManager::new().expect("ProfileManager init"));
    Arc::new(BrowserManager::new(
        BrowserManagerConfig {
            headless: true,
            ..Default::default()
        },
        profile_manager,
    ))
}

/// Helper: validate + execute a script, returning the result JSON.
async fn run_script(
    manager: Arc<BrowserManager>,
    code: &str,
) -> Result<serde_json::Value, String> {
    let validation = code_mode::validate_script(code)?;
    assert!(validation.is_valid);
    code_mode::execute_script(manager, code, &validation.approval_token, None).await
}

// ---------------------------------------------------------------------------
// Test 1: Simple connectivity — navigate to example.com, extract heading
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_simple_navigate_and_extract() {
    let manager = test_manager();

    let code = r##"
        await api.post("/navigate", { url: "https://example.com" });
        const heading = await api.post("/get_text", { selector: "h1" });
        const url = await api.get("/url");
        return { heading: heading, url: url };
    "##;

    let result = run_script(manager, code).await.expect("script should succeed");
    let inner = &result["result"];

    // example.com heading is "Example Domain"
    let heading_text = inner["heading"]["text"].as_str().unwrap_or("");
    assert!(
        heading_text.contains("Example Domain"),
        "Expected 'Example Domain', got: {}",
        heading_text
    );

    let url = inner["url"]["url"].as_str().unwrap_or("");
    assert!(
        url.contains("example.com"),
        "Expected URL containing 'example.com', got: {}",
        url
    );

    assert!(result["api_calls"].as_u64().unwrap() >= 3);
}

// ---------------------------------------------------------------------------
// Test 2: Screenshot — navigate and take a screenshot, verify base64 output
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_screenshot() {
    let manager = test_manager();

    let code = r##"
        await api.post("/navigate", { url: "https://example.com" });
        const shot = await api.post("/screenshot", {});
        return shot;
    "##;

    let result = run_script(manager, code).await.expect("script should succeed");
    let inner = &result["result"];

    // Screenshot should return base64-encoded data
    let has_base64 = inner["screenshot"].is_string()
        || inner.get("data").map_or(false, |d| d.is_string())
        || inner.is_string();
    assert!(has_base64, "Expected base64 screenshot data, got: {}", inner);
}

// ---------------------------------------------------------------------------
// Test 3: Form filling — fill httpbin.org form fields
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_form_filling() {
    let manager = test_manager();

    let code = r##"
        await api.post("/navigate", { url: "https://httpbin.org/forms/post" });
        await api.post("/wait", { timeout_ms: 2000 });

        await api.post("/fill", { selector: "input[name='custname']", value: "John Doe" });
        await api.post("/fill", { selector: "input[name='custtel']", value: "555-0123" });
        await api.post("/fill", { selector: "input[name='custemail']", value: "john@example.com" });

        await api.post("/click", { selector: "input[name='size'][value='medium']" });

        const name_val = await api.post("/evaluate", {
            expression: "document.querySelector('input[name=\"custname\"]').value"
        });

        return { filled_name: name_val };
    "##;

    let result = run_script(manager, code).await.expect("script should succeed");
    let inner = &result["result"];

    // The evaluate result may be a plain string or wrapped in {"result": "..."}
    let filled = &inner["filled_name"];
    let name = filled
        .as_str()
        .or_else(|| filled["result"].as_str())
        .or_else(|| filled["value"].as_str())
        .unwrap_or("");
    assert!(
        name.contains("John Doe"),
        "Expected 'John Doe' in filled name, got: {:?}",
        filled
    );
}

// ---------------------------------------------------------------------------
// Test 4: Keyboard interaction — navigate and press keys
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_keyboard_press() {
    let manager = test_manager();

    let code = r##"
        await api.post("/navigate", { url: "https://example.com" });

        await api.post("/press_key", { key: "Tab" });
        await api.post("/press_key", { key: "Enter" });

        const url = await api.get("/url");
        return { url: url };
    "##;

    let result = run_script(manager, code).await.expect("script should succeed");
    // Just verify it ran without crashing — key presses on example.com are benign
    assert!(result["api_calls"].as_u64().unwrap() >= 3);
}

// ---------------------------------------------------------------------------
// Test 5: DOM extraction — get full page DOM
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_dom_extraction() {
    let manager = test_manager();

    let code = r##"
        await api.post("/navigate", { url: "https://example.com" });
        const dom = await api.get("/dom");
        return dom;
    "##;

    let result = run_script(manager, code).await.expect("script should succeed");
    let inner = &result["result"];

    let dom_html = inner["dom"].as_str().unwrap_or("");
    assert!(
        dom_html.contains("Example Domain"),
        "DOM should contain 'Example Domain', got {} chars",
        dom_html.len()
    );
    assert!(
        dom_html.contains("<html"),
        "DOM should contain <html tag"
    );
}

// ---------------------------------------------------------------------------
// Test 6: JavaScript evaluation — run JS in page context
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_evaluate_script() {
    let manager = test_manager();

    let code = r##"
        await api.post("/navigate", { url: "https://example.com" });
        const title = await api.post("/evaluate", {
            expression: "document.title"
        });
        const count = await api.post("/evaluate", {
            expression: "document.querySelectorAll('p').length"
        });
        return { title: title, paragraph_count: count };
    "##;

    let result = run_script(manager, code).await.expect("script should succeed");
    let inner = &result["result"];

    let title_val = &inner["title"];
    let title = title_val
        .as_str()
        .or_else(|| title_val["result"].as_str())
        .or_else(|| title_val["value"].as_str())
        .unwrap_or("");
    assert!(
        title.contains("Example Domain"),
        "Expected title containing 'Example Domain', got: {:?}",
        title_val
    );
}

// ---------------------------------------------------------------------------
// Test 7: Multi-step workflow with conditionals
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_conditional_workflow() {
    let manager = test_manager();

    let code = r##"
        await api.post("/navigate", { url: "https://example.com" });
        const heading = await api.post("/get_text", { selector: "h1" });

        let result = "unknown";
        if (heading.text === "Example Domain") {
            result = "found_example";
        }

        return { check: result, heading: heading };
    "##;

    let result = run_script(manager, code).await.expect("script should succeed");
    let inner = &result["result"];

    assert_eq!(
        inner["check"].as_str().unwrap_or(""),
        "found_example",
        "Conditional should have matched 'Example Domain'"
    );
}

// ---------------------------------------------------------------------------
// Test 8: Variables — pass external variables into the script
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_with_variables() {
    let manager = test_manager();

    let code = r##"
        await api.post("/navigate", { url: target_url });
        const heading = await api.post("/get_text", { selector: "h1" });
        return { heading: heading, url_used: target_url };
    "##;

    let validation = code_mode::validate_script(code).expect("should validate");
    let variables = Some(serde_json::json!({
        "target_url": "https://example.com"
    }));

    let result = code_mode::execute_script(
        manager,
        code,
        &validation.approval_token,
        variables,
    )
    .await
    .expect("script should succeed");

    let inner = &result["result"];
    assert_eq!(
        inner["url_used"].as_str().unwrap_or(""),
        "https://example.com"
    );
}

// ---------------------------------------------------------------------------
// Test 9: Page listing — verify at least one page exists
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_list_pages() {
    let manager = test_manager();

    let code = r##"
        await api.post("/navigate", { url: "https://example.com" });
        const pages = await api.get("/pages");
        return pages;
    "##;

    let result = run_script(manager, code).await.expect("script should succeed");
    let inner = &result["result"];

    // Should have at least one page
    let pages = inner["pages"].as_array();
    assert!(
        pages.map_or(false, |p| !p.is_empty()),
        "Expected at least one page, got: {}",
        inner
    );
}

// ---------------------------------------------------------------------------
// Test 10: Hover — hover over element (no crash)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_hover_element() {
    let manager = test_manager();

    let code = r##"
        await api.post("/navigate", { url: "https://example.com" });
        await api.post("/hover", { selector: "h1" });
        return { status: "hovered" };
    "##;

    let result = run_script(manager, code).await.expect("script should succeed");
    assert_eq!(result["result"]["status"].as_str().unwrap_or(""), "hovered");
}
