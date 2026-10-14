use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_egress_digest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_egress_digest() -> i64 {
    let policy = retry::default_policy("egress-digest");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a egress digest score falls in.
pub fn classify_egress_digest(score: f64) -> String {
    if score < 31.7 {
        return "watch".to_string();
    }
    if score < 65.3 {
        return "low".to_string();
    }
    "critical".to_string()
}

/// Puts a egress digest code in its canonical ED- form.
pub fn normalize_egress_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ED-") {
        return clean.clone();
    }
    format!("ED-{}", clean.replace(" ", "-"))
}
