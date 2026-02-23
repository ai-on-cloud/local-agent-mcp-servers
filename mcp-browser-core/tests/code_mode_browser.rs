//! Integration tests for code mode with a real browser.
//!
//! These tests launch a headless Chrome/Edge instance via CDP and execute
//! code mode scripts against a local test server. They are `#[ignore]` by
//! default because they require a Chrome or Edge binary installed.
//!
//! Run with:
//!   cargo test -p mcp-browser-core --test code_mode_browser -- --ignored --test-threads=1
//!
//! Cross-browser: set the BROWSER env var to select the browser:
//!   BROWSER=chrome  (default — auto-detect Chrome)
//!   BROWSER=edge    (platform-specific Edge path)
//!   BROWSER=/absolute/path/to/binary

mod test_helpers;
mod test_server;

use mcp_browser_core::code_mode;
use test_helpers::{preflight_check, run_script, test_manager};
use test_server::TestServer;

// ---------------------------------------------------------------------------
// Test 1: Simple connectivity — navigate and extract heading
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_simple_navigate_and_extract() {
    preflight_check().await;
    let server = TestServer::start().await;
    let manager = test_manager();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{}" }});
        const heading = await api.post("/get_text", {{ selector: "h1" }});
        const url = await api.get("/url");
        return {{ heading: heading, url: url }};
    "##,
        server.url("simple.html")
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    let inner = &result["result"];

    let heading_text = inner["heading"]["text"].as_str().unwrap_or("");
    assert!(
        heading_text.contains("Test Page Heading"),
        "Expected 'Test Page Heading', got: {}",
        heading_text
    );

    let url = inner["url"]["url"].as_str().unwrap_or("");
    assert!(
        url.contains("simple.html"),
        "Expected URL containing 'simple.html', got: {}",
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
    preflight_check().await;
    let server = TestServer::start().await;
    let manager = test_manager();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{}" }});
        const shot = await api.post("/screenshot", {{}});
        return shot;
    "##,
        server.url("simple.html")
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    let inner = &result["result"];

    let has_base64 = inner["screenshot"].is_string()
        || inner.get("data").map_or(false, |d| d.is_string())
        || inner.is_string();
    assert!(
        has_base64,
        "Expected base64 screenshot data, got: {}",
        inner
    );
}

// ---------------------------------------------------------------------------
// Test 3: Form filling — fill inputs on local form page
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_form_filling() {
    preflight_check().await;
    let server = TestServer::start().await;
    let manager = test_manager();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{}" }});
        await api.post("/fill", {{ selector: "#name", value: "John Doe" }});
        await api.post("/fill", {{ selector: "#email", value: "john@example.com" }});
        await api.post("/fill", {{ selector: "#message", value: "Hello world" }});

        const name_val = await api.post("/evaluate", {{
            expression: "document.getElementById('name').value"
        }});

        return {{ filled_name: name_val }};
    "##,
        server.url("form.html")
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    let inner = &result["result"];

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
    preflight_check().await;
    let server = TestServer::start().await;
    let manager = test_manager();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{}" }});
        await api.post("/press_key", {{ key: "Tab" }});
        await api.post("/press_key", {{ key: "Enter" }});
        const url = await api.get("/url");
        return {{ url: url }};
    "##,
        server.url("simple.html")
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    assert!(result["api_calls"].as_u64().unwrap() >= 3);
}

// ---------------------------------------------------------------------------
// Test 5: DOM extraction — get full page DOM
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_dom_extraction() {
    preflight_check().await;
    let server = TestServer::start().await;
    let manager = test_manager();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{}" }});
        const dom = await api.get("/dom");
        return dom;
    "##,
        server.url("simple.html")
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    let inner = &result["result"];

    let dom_html = inner["dom"].as_str().unwrap_or("");
    assert!(
        dom_html.contains("Test Page Heading"),
        "DOM should contain 'Test Page Heading', got {} chars",
        dom_html.len()
    );
    assert!(dom_html.contains("<html"), "DOM should contain <html tag");
}

// ---------------------------------------------------------------------------
// Test 6: JavaScript evaluation — run JS in page context
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_evaluate_script() {
    preflight_check().await;
    let server = TestServer::start().await;
    let manager = test_manager();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{}" }});
        const title = await api.post("/evaluate", {{
            expression: "document.title"
        }});
        const count = await api.post("/evaluate", {{
            expression: "document.querySelectorAll('a').length"
        }});
        return {{ title: title, link_count: count }};
    "##,
        server.url("simple.html")
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    let inner = &result["result"];

    let title_val = &inner["title"];
    let title = title_val
        .as_str()
        .or_else(|| title_val["result"].as_str())
        .or_else(|| title_val["value"].as_str())
        .unwrap_or("");
    assert!(
        title.contains("Simple Test Page"),
        "Expected title containing 'Simple Test Page', got: {:?}",
        title_val
    );
}

// ---------------------------------------------------------------------------
// Test 7: Multi-step workflow with conditionals
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_conditional_workflow() {
    preflight_check().await;
    let server = TestServer::start().await;
    let manager = test_manager();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{}" }});
        const heading = await api.post("/get_text", {{ selector: "h1" }});

        let result = "unknown";
        if (heading.text === "Test Page Heading") {{
            result = "found_heading";
        }}

        return {{ check: result, heading: heading }};
    "##,
        server.url("simple.html")
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    let inner = &result["result"];

    assert_eq!(
        inner["check"].as_str().unwrap_or(""),
        "found_heading",
        "Conditional should have matched 'Test Page Heading'"
    );
}

// ---------------------------------------------------------------------------
// Test 8: Variables — pass external variables into the script
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_with_variables() {
    preflight_check().await;
    let server = TestServer::start().await;
    let manager = test_manager();

    let code = r##"
        await api.post("/navigate", { url: target_url });
        const heading = await api.post("/get_text", { selector: "h1" });
        return { heading: heading, url_used: target_url };
    "##;

    let validation = code_mode::validate_script(code).expect("should validate");
    let variables = Some(serde_json::json!({
        "target_url": server.url("simple.html")
    }));

    let result = code_mode::execute_script(manager, code, &validation.approval_token, variables)
        .await
        .expect("script should succeed");

    let inner = &result["result"];
    let url_used = inner["url_used"].as_str().unwrap_or("");
    assert!(
        url_used.contains("simple.html"),
        "Expected URL containing 'simple.html', got: {}",
        url_used
    );
}

// ---------------------------------------------------------------------------
// Test 9: Page listing — verify at least one page exists
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_list_pages() {
    preflight_check().await;
    let server = TestServer::start().await;
    let manager = test_manager();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{}" }});
        const pages = await api.get("/pages");
        return pages;
    "##,
        server.url("simple.html")
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    let inner = &result["result"];

    let pages = inner["pages"].as_array();
    assert!(
        pages.map_or(false, |p| !p.is_empty()),
        "Expected at least one page, got: {}",
        inner
    );
}

// ---------------------------------------------------------------------------
// Test 10: Hover — hover over element
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_hover_element() {
    preflight_check().await;
    let server = TestServer::start().await;
    let manager = test_manager();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{}" }});
        await api.post("/hover", {{ selector: "#hoverable" }});
        return {{ status: "hovered" }};
    "##,
        server.url("simple.html")
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    assert_eq!(result["result"]["status"].as_str().unwrap_or(""), "hovered");
}

// ---------------------------------------------------------------------------
// Test 11: Table extraction — extract_table on table.html
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_table_extraction() {
    preflight_check().await;
    let server = TestServer::start().await;
    let manager = test_manager();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{}" }});
        const table = await api.post("/extract_table", {{ selector: "#data-table" }});
        return table;
    "##,
        server.url("table.html")
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    let inner = &result["result"];

    // Check headers
    let headers = &inner["headers"];
    assert!(headers.is_array(), "Expected headers array, got: {}", inner);
    let headers_arr = headers.as_array().unwrap();
    assert_eq!(headers_arr.len(), 3, "Expected 3 headers");
    assert_eq!(headers_arr[0].as_str().unwrap_or(""), "Name");
    assert_eq!(headers_arr[1].as_str().unwrap_or(""), "Age");
    assert_eq!(headers_arr[2].as_str().unwrap_or(""), "City");

    // Check rows
    let rows = &inner["rows"];
    assert!(rows.is_array(), "Expected rows array, got: {}", inner);
    let rows_arr = rows.as_array().unwrap();
    assert_eq!(rows_arr.len(), 3, "Expected 3 rows");
}

// ---------------------------------------------------------------------------
// Test 12: Dynamic wait — wait for delayed element
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_dynamic_wait() {
    preflight_check().await;
    let server = TestServer::start().await;
    let manager = test_manager();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{}" }});
        await api.post("/wait", {{ selector: "#delayed-element", timeout_ms: 5000 }});
        const text = await api.post("/get_text", {{ selector: "#delayed-element" }});
        return text;
    "##,
        server.url("dynamic.html")
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    let inner = &result["result"];

    let text = inner["text"].as_str().unwrap_or("");
    assert!(
        text.contains("appeared after 500ms"),
        "Expected delayed element text, got: {}",
        text
    );
}

// ---------------------------------------------------------------------------
// Test 13: Form submit + evaluate result
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_form_submit_evaluate() {
    preflight_check().await;
    let server = TestServer::start().await;
    let manager = test_manager();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{}" }});
        await api.post("/fill", {{ selector: "#name", value: "Alice" }});
        await api.post("/fill", {{ selector: "#email", value: "alice@test.com" }});
        await api.post("/fill", {{ selector: "#message", value: "Hi there" }});
        await api.post("/click", {{ selector: "#submit-btn" }});

        // Small wait for JS to update the DOM
        await api.post("/wait", {{ timeout_ms: 500 }});

        const result_text = await api.post("/evaluate", {{
            expression: "document.getElementById('result').textContent"
        }});
        return {{ result_text: result_text }};
    "##,
        server.url("form.html")
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    let inner = &result["result"];

    let result_text = &inner["result_text"];
    let text = result_text
        .as_str()
        .or_else(|| result_text["result"].as_str())
        .or_else(|| result_text["value"].as_str())
        .unwrap_or("");
    assert!(
        text.contains("Alice"),
        "Expected submit result to contain 'Alice', got: {:?}",
        result_text
    );
}
