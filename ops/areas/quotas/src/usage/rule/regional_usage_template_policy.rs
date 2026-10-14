use super::*;

use acme_kit::numbers;

/// Puts a regional usage template code in its canonical RUT- form.
pub fn normalize_regional_usage_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RUT-") {
        return clean.clone();
    }
    format!("RUT-{}", clean.replace(" ", "-"))
}

pub fn regional_usage_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn regional_usage_template_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.79;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
