use super::*;

use acme_kit::numbers;

pub fn plan_policy_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

pub fn plan_policy_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.53;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a plan policy code in its canonical PP- form.
pub fn normalize_plan_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PP-") {
        return clean.clone();
    }
    format!("PP-{}", clean.replace(" ", "-"))
}
