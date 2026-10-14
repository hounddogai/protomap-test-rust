use super::*;

use acme_kit::numbers;

pub fn regional_trust_domain_notice_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn regional_trust_domain_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 11704.0, in order.
pub fn regional_trust_domain_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 11704.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a regional trust domain notice score falls in.
pub fn classify_regional_trust_domain_notice(score: f64) -> String {
    if score < 70.1 {
        return "ok".to_string();
    }
    if score < 73.2 {
        return "medium".to_string();
    }
    "elevated".to_string()
}
