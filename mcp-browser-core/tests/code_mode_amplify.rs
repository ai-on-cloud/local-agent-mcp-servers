//! Integration tests for code mode against the deployed Amplify test site.
//!
//! These tests target the static SPA hosted on AWS Amplify, exercising
//! real-world patterns: SPA navigation, client-side search, dynamic content,
//! and multi-step workflows.
//!
//! All tests are `#[ignore]` — run explicitly with:
//!   AMPLIFY_URL=https://<your-site> cargo test -p mcp-browser-core --test code_mode_amplify -- --ignored --test-threads=1
//!
//! Cross-browser: set BROWSER env var (chrome, edge, or absolute path).

mod test_helpers;

use test_helpers::{preflight_check, run_script, test_manager};

/// Read the Amplify site base URL from the environment.
/// Falls back to localhost:5173 for local dev server testing.
fn amplify_url() -> String {
    std::env::var("AMPLIFY_URL").unwrap_or_else(|_| "http://localhost:5173".to_string())
}

// ---------------------------------------------------------------------------
// Test 1: Navigate to home page and verify heading
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_spa_navigate_home() {
    preflight_check().await;
    let manager = test_manager();
    let base = amplify_url();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{base}" }});
        const heading = await api.post("/get_text", {{ selector: "[data-testid='heading-home']" }});
        const url = await api.get("/url");
        return {{ heading: heading, url: url }};
    "##
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    let inner = &result["result"];

    let heading_text = inner["heading"]["text"].as_str().unwrap_or("");
    assert!(
        heading_text.contains("LocalAgent Test Site"),
        "Expected home heading, got: {}",
        heading_text
    );
}

// ---------------------------------------------------------------------------
// Test 2: Client-side routing — click nav link to products
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_spa_client_routing() {
    preflight_check().await;
    let manager = test_manager();
    let base = amplify_url();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{base}" }});
        await api.post("/click", {{ selector: "[data-testid='nav-products']" }});
        await api.post("/wait", {{ selector: "[data-testid='search-input']", timeout_ms: 5000 }});
        const url = await api.get("/url");
        const heading = await api.post("/get_text", {{ selector: "[data-testid='heading-products']" }});
        return {{ url: url, heading: heading }};
    "##
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    let inner = &result["result"];

    let url = inner["url"]["url"].as_str().unwrap_or("");
    assert!(
        url.contains("/products"),
        "Expected URL containing '/products', got: {}",
        url
    );

    let heading_text = inner["heading"]["text"].as_str().unwrap_or("");
    assert!(
        heading_text.contains("Products"),
        "Expected 'Products' heading, got: {}",
        heading_text
    );
}

// ---------------------------------------------------------------------------
// Test 3: Search products — fill input, click search, check results
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_search_products() {
    preflight_check().await;
    let manager = test_manager();
    let base = amplify_url();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{base}/products" }});
        await api.post("/wait", {{ selector: "[data-testid='search-input']", timeout_ms: 5000 }});
        await api.post("/fill", {{ selector: "[data-testid='search-input']", value: "widget" }});
        await api.post("/click", {{ selector: "[data-testid='search-button']" }});
        await api.post("/wait", {{ timeout_ms: 500 }});
        const count = await api.post("/get_text", {{ selector: "[data-testid='results-count']" }});
        return {{ count: count }};
    "##
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    let inner = &result["result"];

    let count_text = inner["count"]["text"].as_str().unwrap_or("");
    // "widget" should match Alpha Widget, Gamma Widget, Eta Widget → 3 results
    assert!(
        count_text.contains("3"),
        "Expected 3 results for 'widget' search, got: {}",
        count_text
    );
}

// ---------------------------------------------------------------------------
// Test 4: Search with no results
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_search_no_results() {
    preflight_check().await;
    let manager = test_manager();
    let base = amplify_url();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{base}/products" }});
        await api.post("/wait", {{ selector: "[data-testid='search-input']", timeout_ms: 5000 }});
        await api.post("/fill", {{ selector: "[data-testid='search-input']", value: "zzzznonexistent" }});
        await api.post("/click", {{ selector: "[data-testid='search-button']" }});
        await api.post("/wait", {{ timeout_ms: 500 }});
        const count = await api.post("/get_text", {{ selector: "[data-testid='results-count']" }});
        return {{ count: count }};
    "##
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    let inner = &result["result"];

    let count_text = inner["count"]["text"].as_str().unwrap_or("");
    assert!(
        count_text.starts_with("0"),
        "Expected 0 results for nonsense search, got: {}",
        count_text
    );
}

// ---------------------------------------------------------------------------
// Test 5: Product detail flow — navigate to product, check detail page
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_product_detail_flow() {
    preflight_check().await;
    let manager = test_manager();
    let base = amplify_url();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{base}/products" }});
        await api.post("/wait", {{ selector: "[data-testid='product-link-1']", timeout_ms: 5000 }});
        await api.post("/click", {{ selector: "[data-testid='product-link-1']" }});
        await api.post("/wait", {{ selector: "[data-testid='product-name']", timeout_ms: 5000 }});
        const name = await api.post("/get_text", {{ selector: "[data-testid='product-name']" }});
        const price = await api.post("/get_text", {{ selector: "[data-testid='product-price']" }});
        const desc = await api.post("/get_text", {{ selector: "[data-testid='product-description']" }});
        return {{ name: name, price: price, description: desc }};
    "##
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    let inner = &result["result"];

    let name = inner["name"]["text"].as_str().unwrap_or("");
    assert!(
        name.contains("Alpha Widget"),
        "Expected 'Alpha Widget', got: {}",
        name
    );

    let price = inner["price"]["text"].as_str().unwrap_or("");
    assert!(
        price.contains("29.99"),
        "Expected price containing '29.99', got: {}",
        price
    );

    let desc = inner["description"]["text"].as_str().unwrap_or("");
    assert!(!desc.is_empty(), "Expected non-empty description");
}

// ---------------------------------------------------------------------------
// Test 6: Extract results table
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_extract_results_table() {
    preflight_check().await;
    let manager = test_manager();
    let base = amplify_url();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{base}/products" }});
        await api.post("/wait", {{ selector: "[data-testid='results-table']", timeout_ms: 5000 }});
        const table = await api.post("/extract_table", {{ selector: "[data-testid='results-table']" }});
        return table;
    "##
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    let inner = &result["result"];

    // Check headers
    let headers = &inner["headers"];
    assert!(headers.is_array(), "Expected headers array, got: {}", inner);
    let headers_arr = headers.as_array().unwrap();
    assert_eq!(
        headers_arr.len(),
        3,
        "Expected 3 headers (Name, Category, Price)"
    );
    assert_eq!(headers_arr[0].as_str().unwrap_or(""), "Name");
    assert_eq!(headers_arr[1].as_str().unwrap_or(""), "Category");
    assert_eq!(headers_arr[2].as_str().unwrap_or(""), "Price");

    // Check rows — 10 products by default (no search filter)
    let rows = &inner["rows"];
    assert!(rows.is_array(), "Expected rows array, got: {}", inner);
    let rows_arr = rows.as_array().unwrap();
    assert_eq!(rows_arr.len(), 10, "Expected 10 product rows");
}

// ---------------------------------------------------------------------------
// Test 7: Dynamic content — wait for delayed element
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_dynamic_content() {
    preflight_check().await;
    let manager = test_manager();
    let base = amplify_url();

    let code = format!(
        r##"
        await api.post("/navigate", {{ url: "{base}/dynamic" }});
        await api.post("/wait", {{ selector: "[data-testid='delayed-content']", timeout_ms: 3000 }});
        const text = await api.post("/get_text", {{ selector: "[data-testid='delayed-content']" }});
        return text;
    "##
    );

    let result = run_script(manager, &code)
        .await
        .expect("script should succeed");
    let inner = &result["result"];

    let text = inner["text"].as_str().unwrap_or("");
    assert!(
        text.contains("appeared after 500ms"),
        "Expected delayed content text, got: {}",
        text
    );
}

// ---------------------------------------------------------------------------
// Test 8: Multi-step workflow — full round-trip
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_multi_step_workflow() {
    preflight_check().await;
    let manager = test_manager();
    let base = amplify_url();

    let code = format!(
        r##"
        // Step 1: Start at home
        await api.post("/navigate", {{ url: "{base}" }});
        const home_heading = await api.post("/get_text", {{ selector: "[data-testid='heading-home']" }});

        // Step 2: Navigate to products via SPA link
        await api.post("/click", {{ selector: "[data-testid='nav-products']" }});
        await api.post("/wait", {{ selector: "[data-testid='search-input']", timeout_ms: 5000 }});

        // Step 3: Search for a product
        await api.post("/fill", {{ selector: "[data-testid='search-input']", value: "sensor" }});
        await api.post("/click", {{ selector: "[data-testid='search-button']" }});
        await api.post("/wait", {{ timeout_ms: 500 }});
        const search_count = await api.post("/get_text", {{ selector: "[data-testid='results-count']" }});

        // Step 4: Click into product detail
        await api.post("/click", {{ selector: "[data-testid='product-link-5']" }});
        await api.post("/wait", {{ selector: "[data-testid='product-name']", timeout_ms: 5000 }});
        const detail_name = await api.post("/get_text", {{ selector: "[data-testid='product-name']" }});

        // Step 5: Go back to products
        await api.post("/click", {{ selector: "[data-testid='back-link']" }});
        await api.post("/wait", {{ selector: "[data-testid='search-input']", timeout_ms: 5000 }});
        const back_url = await api.get("/url");

        return {{
            home_heading: home_heading,
            search_count: search_count,
            detail_name: detail_name,
            back_url: back_url
        }};
    "##
    );

    let result = run_script(manager, &code)
        .await
        .expect("multi-step workflow should succeed");
    let inner = &result["result"];

    // Verify home page was reached
    let home = inner["home_heading"]["text"].as_str().unwrap_or("");
    assert!(
        home.contains("LocalAgent Test Site"),
        "Expected home heading, got: {}",
        home
    );

    // Verify search found sensors (Epsilon Sensor, Iota Sensor → 2)
    let count = inner["search_count"]["text"].as_str().unwrap_or("");
    assert!(
        count.contains("2"),
        "Expected 2 sensor results, got: {}",
        count
    );

    // Verify product detail
    let detail = inner["detail_name"]["text"].as_str().unwrap_or("");
    assert!(
        detail.contains("Epsilon Sensor"),
        "Expected 'Epsilon Sensor' detail, got: {}",
        detail
    );

    // Verify we navigated back to products
    let back_url = inner["back_url"]["url"].as_str().unwrap_or("");
    assert!(
        back_url.contains("/products"),
        "Expected to be back on /products, got: {}",
        back_url
    );
}
