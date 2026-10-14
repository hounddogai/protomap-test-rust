use super::*;

use acme_kit::numbers;

/// Puts a lenient quote profile code in its canonical LQP- form.
pub fn normalize_lenient_quote_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LQP-") {
        return clean.clone();
    }
    format!("LQP-{}", clean.replace(" ", "-"))
}

pub fn lenient_quote_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.12;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn lenient_quote_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
