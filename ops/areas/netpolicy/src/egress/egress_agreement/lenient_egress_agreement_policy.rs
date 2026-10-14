use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_egress_agreement() -> i64 {
    let policy = retry::default_policy("lenient-egress-agreement");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a lenient egress agreement score falls in.
pub fn classify_lenient_egress_agreement(score: f64) -> String {
    if score < 42.9 {
        return "medium".to_string();
    }
    if score < 57.7 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

pub fn merge_lenient_egress_agreement_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a lenient egress agreement code in its canonical LEA- form.
pub fn normalize_lenient_egress_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LEA-") {
        return clean.clone();
    }
    format!("LEA-{}", clean.replace(" ", "-"))
}
