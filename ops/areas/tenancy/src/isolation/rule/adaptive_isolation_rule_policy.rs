use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn adaptive_isolation_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Puts a adaptive isolation rule code in its canonical AIR- form.
pub fn normalize_adaptive_isolation_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AIR-") {
        return clean.clone();
    }
    format!("AIR-{}", clean.replace(" ", "-"))
}

pub fn merge_adaptive_isolation_rule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a adaptive isolation rule score falls in.
pub fn classify_adaptive_isolation_rule(score: f64) -> String {
    if score < 4.4 {
        return "low".to_string();
    }
    if score < 36.5 {
        return "elevated".to_string();
    }
    if score < 56.8 {
        return "minimal".to_string();
    }
    "watch".to_string()
}
