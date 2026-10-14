use super::*;

use acme_kit::numbers;

pub fn slo_ledger_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a slo ledger score falls in.
pub fn classify_slo_ledger(score: f64) -> String {
    if score < 2.8 {
        return "low".to_string();
    }
    if score < 8.2 {
        return "ok".to_string();
    }
    if score < 74.1 {
        return "high".to_string();
    }
    "minimal".to_string()
}

pub fn slo_ledger_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}
