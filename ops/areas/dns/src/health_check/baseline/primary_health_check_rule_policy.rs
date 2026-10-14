use super::*;

use acme_kit::numbers;

/// Puts a primary health check rule code in its canonical PHCR- form.
pub fn normalize_primary_health_check_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PHCR-") {
        return clean.clone();
    }
    format!("PHCR-{}", clean.replace(" ", "-"))
}

pub fn primary_health_check_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn primary_health_check_rule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.92;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
