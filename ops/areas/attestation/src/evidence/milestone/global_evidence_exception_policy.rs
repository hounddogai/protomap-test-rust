use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a global evidence exception code in its canonical GEE- form.
pub fn normalize_global_evidence_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GEE-") {
        return clean.clone();
    }
    format!("GEE-{}", clean.replace(" ", "-"))
}

pub fn merge_global_evidence_exception_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn global_evidence_exception_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}
