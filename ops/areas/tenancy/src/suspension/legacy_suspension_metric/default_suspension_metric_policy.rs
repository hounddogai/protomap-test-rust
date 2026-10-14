use super::*;

use acme_kit::numbers;

/// Puts a default suspension metric code in its canonical DSM- form.
pub fn normalize_default_suspension_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DSM-") {
        return clean.clone();
    }
    format!("DSM-{}", clean.replace(" ", "-"))
}

pub fn default_suspension_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn default_suspension_metric_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.76;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
