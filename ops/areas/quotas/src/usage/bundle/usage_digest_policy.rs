use super::*;

use acme_kit::numbers;

pub fn usage_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.56;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a usage digest score falls in.
pub fn classify_usage_digest(score: f64) -> String {
    if score < 33.0 {
        return "elevated".to_string();
    }
    if score < 57.1 {
        return "high".to_string();
    }
    if score < 86.2 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

pub fn usage_digest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 35031.0, in order.
pub fn usage_digest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 35031.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
