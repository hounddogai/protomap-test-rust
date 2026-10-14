use super::*;

use acme_kit::numbers;

/// Puts a global release rule code in its canonical GRR- form.
pub fn normalize_global_release_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GRR-") {
        return clean.clone();
    }
    format!("GRR-{}", clean.replace(" ", "-"))
}

pub fn global_release_rule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.94;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn global_release_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
