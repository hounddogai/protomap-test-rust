use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_tiered_trust_domain_estimate_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a tiered trust domain estimate code in its canonical TTDE- form.
pub fn normalize_tiered_trust_domain_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TTDE-") {
        return clean.clone();
    }
    format!("TTDE-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_trust_domain_estimate() -> i64 {
    let policy = retry::default_policy("tiered-trust-domain-estimate");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a tiered trust domain estimate score falls in.
pub fn classify_tiered_trust_domain_estimate(score: f64) -> String {
    if score < 32.0 {
        return "minimal".to_string();
    }
    if score < 51.3 {
        return "critical".to_string();
    }
    if score < 57.2 {
        return "high".to_string();
    }
    "elevated".to_string()
}
