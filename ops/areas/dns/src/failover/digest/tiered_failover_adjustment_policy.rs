use super::*;

use acme_kit::numbers;

/// Puts a tiered failover adjustment code in its canonical TFA- form.
pub fn normalize_tiered_failover_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TFA-") {
        return clean.clone();
    }
    format!("TFA-{}", clean.replace(" ", "-"))
}

pub fn tiered_failover_adjustment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.29;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
