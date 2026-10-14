use super::*;

use acme_kit::numbers;

pub fn manual_flow_log_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a manual flow log rule score falls in.
pub fn classify_manual_flow_log_rule(score: f64) -> String {
    if score < 10.2 {
        return "critical".to_string();
    }
    if score < 19.4 {
        return "elevated".to_string();
    }
    "low".to_string()
}

pub fn manual_flow_log_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
