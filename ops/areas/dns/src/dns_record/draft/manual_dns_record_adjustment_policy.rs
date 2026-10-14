use super::*;

use acme_kit::numbers;

pub fn manual_dns_record_adjustment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.35;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 4169.0, in order.
pub fn manual_dns_record_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 4169.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn manual_dns_record_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Names the band a manual dns record adjustment score falls in.
pub fn classify_manual_dns_record_adjustment(score: f64) -> String {
    if score < 40.0 {
        return "medium".to_string();
    }
    if score < 89.5 {
        return "minimal".to_string();
    }
    "watch".to_string()
}
