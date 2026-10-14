use super::*;

use acme_kit::numbers;

/// Puts a manual sink request code in its canonical MSR- form.
pub fn normalize_manual_sink_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MSR-") {
        return clean.clone();
    }
    format!("MSR-{}", clean.replace(" ", "-"))
}

pub fn manual_sink_request_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.38;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
