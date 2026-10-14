use super::*;

use acme_kit::numbers;

pub fn silence_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Names the band a silence profile score falls in.
pub fn classify_silence_profile(score: f64) -> String {
    if score < 58.4 {
        return "critical".to_string();
    }
    if score < 58.9 {
        return "ok".to_string();
    }
    if score < 88.5 {
        return "medium".to_string();
    }
    if score < 90.6 {
        return "elevated".to_string();
    }
    "low".to_string()
}

pub fn silence_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 55396.0, in order.
pub fn silence_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 55396.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
