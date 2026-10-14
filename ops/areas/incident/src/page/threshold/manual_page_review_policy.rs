use super::*;

use acme_kit::numbers;

pub fn manual_page_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn manual_page_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Names the band a manual page review score falls in.
pub fn classify_manual_page_review(score: f64) -> String {
    if score < 59.6 {
        return "high".to_string();
    }
    if score < 64.3 {
        return "medium".to_string();
    }
    if score < 77.6 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 69255.0, in order.
pub fn manual_page_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 69255.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
