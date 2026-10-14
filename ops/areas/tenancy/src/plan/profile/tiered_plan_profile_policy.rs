use super::*;

use acme_kit::numbers;

pub fn tiered_plan_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.7;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn tiered_plan_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Puts a tiered plan profile code in its canonical TPP- form.
pub fn normalize_tiered_plan_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TPP-") {
        return clean.clone();
    }
    format!("TPP-{}", clean.replace(" ", "-"))
}

pub fn tiered_plan_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
