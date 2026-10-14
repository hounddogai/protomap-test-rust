use super::*;

use acme_kit::numbers;

pub fn lenient_egress_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Counts how many amounts fit in a budget of 2862.0, in order.
pub fn lenient_egress_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 2862.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a lenient egress profile score falls in.
pub fn classify_lenient_egress_profile(score: f64) -> String {
    if score < 14.2 {
        return "minimal".to_string();
    }
    if score < 77.1 {
        return "medium".to_string();
    }
    if score < 91.3 {
        return "low".to_string();
    }
    "ok".to_string()
}

pub fn lenient_egress_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
