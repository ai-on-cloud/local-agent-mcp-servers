//! Press a keyboard key, optionally with modifiers.

use crate::browser::BrowserManager;
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchKeyEventParams, DispatchKeyEventType,
};
use pmcp::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[schemars(deny_unknown_fields)]
pub struct PressKeyInput {
    /// Key to press (e.g. "Enter", "Tab", "Escape", "a", "Control+a", "Shift+Tab")
    #[validate(length(min = 1))]
    #[schemars(
        description = "Key to press. Examples: 'Enter', 'Tab', 'Escape', 'Backspace', 'ArrowDown', 'a', 'Control+a', 'Shift+Tab', 'Meta+c'"
    )]
    pub key: String,

    /// CSS selector of element to focus before pressing key (optional)
    #[schemars(description = "CSS selector of element to focus before pressing the key (optional)")]
    pub selector: Option<String>,
}

/// Parse modifier+key string like "Control+a" into (modifier_flags, key_name).
fn parse_key_combo(combo: &str) -> (i64, &str) {
    let parts: Vec<&str> = combo.split('+').collect();
    if parts.len() == 1 {
        return (0, parts[0]);
    }

    let mut modifiers: i64 = 0;
    for &part in &parts[..parts.len() - 1] {
        match part.to_lowercase().as_str() {
            "alt" => modifiers |= 1,
            "ctrl" | "control" => modifiers |= 2,
            "meta" | "cmd" | "command" => modifiers |= 4,
            "shift" => modifiers |= 8,
            _ => {}
        }
    }

    (modifiers, parts[parts.len() - 1])
}

/// Map common key names to (key, code, keyCode) for CDP.
fn key_definition(name: &str) -> (&str, String, i64) {
    match name {
        "Enter" | "Return" => ("Enter", "Enter".into(), 13),
        "Tab" => ("Tab", "Tab".into(), 9),
        "Escape" | "Esc" => ("Escape", "Escape".into(), 27),
        "Backspace" => ("Backspace", "Backspace".into(), 8),
        "Delete" => ("Delete", "Delete".into(), 46),
        "Space" | " " => (" ", "Space".into(), 32),
        "ArrowUp" => ("ArrowUp", "ArrowUp".into(), 38),
        "ArrowDown" => ("ArrowDown", "ArrowDown".into(), 40),
        "ArrowLeft" => ("ArrowLeft", "ArrowLeft".into(), 37),
        "ArrowRight" => ("ArrowRight", "ArrowRight".into(), 39),
        "Home" => ("Home", "Home".into(), 36),
        "End" => ("End", "End".into(), 35),
        "PageUp" => ("PageUp", "PageUp".into(), 33),
        "PageDown" => ("PageDown", "PageDown".into(), 34),
        "F1" => ("F1", "F1".into(), 112),
        "F2" => ("F2", "F2".into(), 113),
        "F3" => ("F3", "F3".into(), 114),
        "F4" => ("F4", "F4".into(), 115),
        "F5" => ("F5", "F5".into(), 116),
        "F6" => ("F6", "F6".into(), 117),
        "F7" => ("F7", "F7".into(), 118),
        "F8" => ("F8", "F8".into(), 119),
        "F9" => ("F9", "F9".into(), 120),
        "F10" => ("F10", "F10".into(), 121),
        "F11" => ("F11", "F11".into(), 122),
        "F12" => ("F12", "F12".into(), 123),
        // Single character
        c if c.len() == 1 => {
            let ch = c.chars().next().unwrap();
            if ch.is_ascii_alphabetic() {
                let upper = ch.to_ascii_uppercase();
                let code = format!("Key{}", upper);
                let key_code = upper as i64;
                // Return key as-is (lowercase preserved)
                return (name, code, key_code);
            }
            if ch.is_ascii_digit() {
                let code = format!("Digit{}", ch);
                let key_code = ch as i64;
                return (name, code, key_code);
            }
            (name, String::new(), 0)
        }
        // Fallback: just pass the key name through
        other => (other, String::new(), 0),
    }
}

pub async fn execute(
    manager: &Arc<BrowserManager>,
    input: PressKeyInput,
) -> Result<serde_json::Value, Error> {
    input
        .validate()
        .map_err(|e| Error::validation(format!("Validation failed: {}", e)))?;

    let page = manager
        .page()
        .await
        .map_err(|e| Error::internal(format!("Browser error: {}", e)))?;

    // Focus element if selector provided
    if let Some(ref selector) = input.selector {
        let element = page
            .find_element(selector)
            .await
            .map_err(|e| Error::internal(format!("Element not found '{}': {}", selector, e)))?;
        element
            .click()
            .await
            .map_err(|e| Error::internal(format!("Failed to focus '{}': {}", selector, e)))?;
    }

    let (modifiers, key_name) = parse_key_combo(&input.key);
    let (key, code, key_code) = key_definition(key_name);

    // Build keyDown params
    let mut down = DispatchKeyEventParams::new(DispatchKeyEventType::KeyDown);
    down.key = Some(key.to_string());
    if !code.is_empty() {
        down.code = Some(code.clone());
    }
    if key_code != 0 {
        down.windows_virtual_key_code = Some(key_code);
    }
    if modifiers != 0 {
        down.modifiers = Some(modifiers);
    }

    // Dispatch keyDown
    page.execute(down)
        .await
        .map_err(|e| Error::internal(format!("Key down failed: {}", e)))?;

    // Build keyUp params
    let mut up = DispatchKeyEventParams::new(DispatchKeyEventType::KeyUp);
    up.key = Some(key.to_string());
    if !code.is_empty() {
        up.code = Some(code);
    }
    if key_code != 0 {
        up.windows_virtual_key_code = Some(key_code);
    }
    if modifiers != 0 {
        up.modifiers = Some(modifiers);
    }

    // Dispatch keyUp
    page.execute(up)
        .await
        .map_err(|e| Error::internal(format!("Key up failed: {}", e)))?;

    Ok(json!({
        "status": "pressed",
        "key": input.key
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_combo_simple() {
        let (mods, key) = parse_key_combo("Enter");
        assert_eq!(mods, 0);
        assert_eq!(key, "Enter");
    }

    #[test]
    fn test_parse_key_combo_with_modifier() {
        let (mods, key) = parse_key_combo("Control+a");
        assert_eq!(mods, 2); // Ctrl
        assert_eq!(key, "a");
    }

    #[test]
    fn test_parse_key_combo_multiple_modifiers() {
        let (mods, key) = parse_key_combo("Control+Shift+Tab");
        assert_eq!(mods, 2 | 8); // Ctrl + Shift
        assert_eq!(key, "Tab");
    }

    #[test]
    fn test_key_definition_enter() {
        let (key, code, kc) = key_definition("Enter");
        assert_eq!(key, "Enter");
        assert_eq!(code, "Enter");
        assert_eq!(kc, 13);
    }

    #[test]
    fn test_key_definition_letter() {
        let (key, code, kc) = key_definition("a");
        assert_eq!(key, "a");
        assert_eq!(code, "KeyA");
        assert_eq!(kc, 65);
    }

    #[test]
    fn test_key_definition_digit() {
        let (key, code, kc) = key_definition("5");
        assert_eq!(key, "5");
        assert_eq!(code, "Digit5");
        assert_eq!(kc, 53);
    }
}
