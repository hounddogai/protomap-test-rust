use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn manual_failover_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Puts a manual failover cycle code in its canonical MFC- form.
pub fn normalize_manual_failover_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MFC-") {
        return clean.clone();
    }
    format!("MFC-{}", clean.replace(" ", "-"))
}

/// Names the band a manual failover cycle score falls in.
pub fn classify_manual_failover_cycle(score: f64) -> String {
    if score < 13.4 {
        return "critical".to_string();
    }
    if score < 19.1 {
        return "minimal".to_string();
    }
    "high".to_string()
}

pub fn merge_manual_failover_cycle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
