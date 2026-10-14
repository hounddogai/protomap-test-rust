use super::*;

use acme_kit::numbers;

pub fn tiered_quote_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

pub fn tiered_quote_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a tiered quote reservation score falls in.
pub fn classify_tiered_quote_reservation(score: f64) -> String {
    if score < 14.5 {
        return "ok".to_string();
    }
    if score < 16.0 {
        return "elevated".to_string();
    }
    if score < 77.6 {
        return "low".to_string();
    }
    if score < 80.8 {
        return "high".to_string();
    }
    "medium".to_string()
}
