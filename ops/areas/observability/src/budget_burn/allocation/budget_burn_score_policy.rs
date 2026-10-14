use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a budget burn score code in its canonical BBS- form.
pub fn normalize_budget_burn_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BBS-") {
        return clean.clone();
    }
    format!("BBS-{}", clean.replace(" ", "-"))
}

pub fn merge_budget_burn_score_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn budget_burn_score_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Names the band a budget burn score score falls in.
pub fn classify_budget_burn_score(score: f64) -> String {
    if score < 73.9 {
        return "minimal".to_string();
    }
    if score < 88.7 {
        return "critical".to_string();
    }
    if score < 97.3 {
        return "medium".to_string();
    }
    "low".to_string()
}
