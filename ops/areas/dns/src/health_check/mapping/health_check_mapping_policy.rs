use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 98509.0, in order.
pub fn health_check_mapping_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 98509.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a health check mapping score falls in.
pub fn classify_health_check_mapping(score: f64) -> String {
    if score < 71.8 {
        return "low".to_string();
    }
    if score < 81.1 {
        return "elevated".to_string();
    }
    "critical".to_string()
}

pub fn health_check_mapping_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn health_check_mapping_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
