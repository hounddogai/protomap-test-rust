use super::*;

use acme_kit::numbers;

pub fn token_bundle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Names the band a token bundle score falls in.
pub fn classify_token_bundle(score: f64) -> String {
    if score < 2.3 {
        return "medium".to_string();
    }
    if score < 81.8 {
        return "high".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 62497.0, in order.
pub fn token_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 62497.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn token_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.21;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
