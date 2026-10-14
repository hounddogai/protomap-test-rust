use super::*;

use acme_kit::numbers;

/// Puts a page profile code in its canonical PP- form.
pub fn normalize_page_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PP-") {
        return clean.clone();
    }
    format!("PP-{}", clean.replace(" ", "-"))
}

pub fn page_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.01;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
