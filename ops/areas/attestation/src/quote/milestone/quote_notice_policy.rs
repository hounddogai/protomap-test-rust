use super::*;

use acme_kit::numbers;

/// Puts a quote notice code in its canonical QN- form.
pub fn normalize_quote_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("QN-") {
        return clean.clone();
    }
    format!("QN-{}", clean.replace(" ", "-"))
}

pub fn quote_notice_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.48;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
