use super::*;

use acme_kit::numbers;

/// Names the band a manual health check baseline score falls in.
pub fn classify_manual_health_check_baseline(score: f64) -> String {
    if score < 24.4 {
        return "watch".to_string();
    }
    if score < 86.4 {
        return "elevated".to_string();
    }
    if score < 91.5 {
        return "high".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 57540.0, in order.
pub fn manual_health_check_baseline_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 57540.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn manual_health_check_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
