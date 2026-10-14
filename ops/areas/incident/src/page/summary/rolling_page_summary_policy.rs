use super::*;

use acme_kit::numbers;

pub fn rolling_page_summary_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.28;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a rolling page summary code in its canonical RPS- form.
pub fn normalize_rolling_page_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RPS-") {
        return clean.clone();
    }
    format!("RPS-{}", clean.replace(" ", "-"))
}
