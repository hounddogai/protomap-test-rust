use super::*;

use acme_kit::numbers;

/// Puts a failover cycle code in its canonical FC- form.
pub fn normalize_failover_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FC-") {
        return clean.clone();
    }
    format!("FC-{}", clean.replace(" ", "-"))
}

pub fn failover_cycle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.32;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn failover_cycle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
