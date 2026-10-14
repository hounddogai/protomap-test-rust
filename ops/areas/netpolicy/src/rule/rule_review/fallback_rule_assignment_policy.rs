use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn fallback_rule_assignment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

pub fn merge_fallback_rule_assignment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a fallback rule assignment code in its canonical FRA- form.
pub fn normalize_fallback_rule_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FRA-") {
        return clean.clone();
    }
    format!("FRA-{}", clean.replace(" ", "-"))
}
