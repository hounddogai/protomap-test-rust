use super::*;

use acme_kit::retry;

/// Names the band a regional dns record segment score falls in.
pub fn classify_regional_dns_record_segment(score: f64) -> String {
    if score < 27.1 {
        return "low".to_string();
    }
    if score < 41.1 {
        return "high".to_string();
    }
    if score < 62.0 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 72973.0, in order.
pub fn regional_dns_record_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 72973.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_dns_record_segment() -> i64 {
    let policy = retry::default_policy("regional-dns-record-segment");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
