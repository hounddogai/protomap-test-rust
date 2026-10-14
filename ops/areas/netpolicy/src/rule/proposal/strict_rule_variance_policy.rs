use super::*;

use acme_kit::numbers;

/// Puts a strict rule variance code in its canonical SRV- form.
pub fn normalize_strict_rule_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SRV-") {
        return clean.clone();
    }
    format!("SRV-{}", clean.replace(" ", "-"))
}

pub fn strict_rule_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn strict_rule_variance_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.28;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
