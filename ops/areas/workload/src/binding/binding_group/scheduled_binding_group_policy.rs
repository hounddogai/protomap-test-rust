use super::*;

use acme_kit::numbers;

/// Puts a scheduled binding group code in its canonical SBG- form.
pub fn normalize_scheduled_binding_group_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SBG-") {
        return clean.clone();
    }
    format!("SBG-{}", clean.replace(" ", "-"))
}

pub fn scheduled_binding_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
