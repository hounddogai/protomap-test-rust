use super::*;

use acme_kit::numbers;

pub fn rolling_region_pin_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a rolling region pin profile score falls in.
pub fn classify_rolling_region_pin_profile(score: f64) -> String {
    if score < 14.2 {
        return "ok".to_string();
    }
    if score < 52.4 {
        return "elevated".to_string();
    }
    if score < 61.0 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 75838.0, in order.
pub fn rolling_region_pin_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 75838.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn rolling_region_pin_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
