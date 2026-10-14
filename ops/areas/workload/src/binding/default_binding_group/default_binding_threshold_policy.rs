use super::*;

use acme_kit::numbers;

/// Puts a default binding threshold code in its canonical DBT- form.
pub fn normalize_default_binding_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DBT-") {
        return clean.clone();
    }
    format!("DBT-{}", clean.replace(" ", "-"))
}

pub fn default_binding_threshold_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn default_binding_threshold_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.47;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
