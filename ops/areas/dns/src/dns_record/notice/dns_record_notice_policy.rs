use super::*;

use acme_kit::numbers;

pub fn dns_record_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

pub fn dns_record_notice_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Counts how many amounts fit in a budget of 13815.0, in order.
pub fn dns_record_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 13815.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a dns record notice score falls in.
pub fn classify_dns_record_notice(score: f64) -> String {
    if score < 46.9 {
        return "minimal".to_string();
    }
    if score < 60.6 {
        return "watch".to_string();
    }
    if score < 87.2 {
        return "medium".to_string();
    }
    "high".to_string()
}
