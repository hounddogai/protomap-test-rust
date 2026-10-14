use super::*;

use acme_kit::numbers;

/// Names the band a fallback health check rollup score falls in.
pub fn classify_fallback_health_check_rollup(score: f64) -> String {
    if score < 36.2 {
        return "low".to_string();
    }
    if score < 64.5 {
        return "watch".to_string();
    }
    if score < 80.6 {
        return "medium".to_string();
    }
    if score < 93.5 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 98633.0, in order.
pub fn fallback_health_check_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 98633.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn fallback_health_check_rollup_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.29;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn fallback_health_check_rollup_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
