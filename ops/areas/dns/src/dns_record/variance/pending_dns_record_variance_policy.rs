use super::*;

use acme_kit::numbers;

/// Names the band a pending dns record variance score falls in.
pub fn classify_pending_dns_record_variance(score: f64) -> String {
    if score < 8.6 {
        return "medium".to_string();
    }
    if score < 20.7 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 57003.0, in order.
pub fn pending_dns_record_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 57003.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn pending_dns_record_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
