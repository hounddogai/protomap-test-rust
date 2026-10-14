use super::*;

use acme_kit::numbers;

/// Puts a egress summary code in its canonical ES- form.
pub fn normalize_egress_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ES-") {
        return clean.clone();
    }
    format!("ES-{}", clean.replace(" ", "-"))
}

pub fn egress_summary_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.8;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
