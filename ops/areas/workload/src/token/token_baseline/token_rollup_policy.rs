use super::*;

use acme_kit::numbers;

pub fn token_rollup_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.58;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a token rollup code in its canonical TR- form.
pub fn normalize_token_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TR-") {
        return clean.clone();
    }
    format!("TR-{}", clean.replace(" ", "-"))
}
