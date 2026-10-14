use super::*;

use acme_kit::numbers;

/// Puts a rolling federation request code in its canonical RFR- form.
pub fn normalize_rolling_federation_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RFR-") {
        return clean.clone();
    }
    format!("RFR-{}", clean.replace(" ", "-"))
}

pub fn rolling_federation_request_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn rolling_federation_request_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.27;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
