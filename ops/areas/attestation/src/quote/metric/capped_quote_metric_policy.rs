use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn capped_quote_metric_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Names the band a capped quote metric score falls in.
pub fn classify_capped_quote_metric(score: f64) -> String {
    if score < 25.4 {
        return "low".to_string();
    }
    if score < 55.5 {
        return "ok".to_string();
    }
    if score < 76.5 {
        return "elevated".to_string();
    }
    if score < 99.8 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

/// Puts a capped quote metric code in its canonical CQM- form.
pub fn normalize_capped_quote_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CQM-") {
        return clean.clone();
    }
    format!("CQM-{}", clean.replace(" ", "-"))
}

pub fn merge_capped_quote_metric_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
