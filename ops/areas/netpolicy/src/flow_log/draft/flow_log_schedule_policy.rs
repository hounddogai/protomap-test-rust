use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn flow_log_schedule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

pub fn flow_log_schedule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn merge_flow_log_schedule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a flow log schedule code in its canonical FLS- form.
pub fn normalize_flow_log_schedule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FLS-") {
        return clean.clone();
    }
    format!("FLS-{}", clean.replace(" ", "-"))
}
