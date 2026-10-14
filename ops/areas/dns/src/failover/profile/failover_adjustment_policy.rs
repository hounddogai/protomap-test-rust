use super::*;

use acme_kit::numbers;

/// Names the band a failover adjustment score falls in.
pub fn classify_failover_adjustment(score: f64) -> String {
    if score < 37.4 {
        return "medium".to_string();
    }
    if score < 38.3 {
        return "critical".to_string();
    }
    if score < 47.5 {
        return "watch".to_string();
    }
    if score < 48.6 {
        return "high".to_string();
    }
    "elevated".to_string()
}

pub fn failover_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

pub fn failover_adjustment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a failover adjustment code in its canonical FA- form.
pub fn normalize_failover_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FA-") {
        return clean.clone();
    }
    format!("FA-{}", clean.replace(" ", "-"))
}
