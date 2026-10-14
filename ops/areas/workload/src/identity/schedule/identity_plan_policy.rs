use super::*;

use acme_kit::numbers;

/// Puts a identity plan code in its canonical IP- form.
pub fn normalize_identity_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("IP-") {
        return clean.clone();
    }
    format!("IP-{}", clean.replace(" ", "-"))
}

pub fn identity_plan_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn identity_plan_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.18;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
