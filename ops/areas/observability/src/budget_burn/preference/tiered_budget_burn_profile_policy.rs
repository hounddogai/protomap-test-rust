use super::*;

use acme_kit::numbers;

pub fn tiered_budget_burn_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

pub fn tiered_budget_burn_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn tiered_budget_burn_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.2;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a tiered budget burn profile score falls in.
pub fn classify_tiered_budget_burn_profile(score: f64) -> String {
    if score < 7.4 {
        return "elevated".to_string();
    }
    if score < 9.4 {
        return "ok".to_string();
    }
    if score < 28.1 {
        return "low".to_string();
    }
    if score < 75.6 {
        return "high".to_string();
    }
    "medium".to_string()
}
