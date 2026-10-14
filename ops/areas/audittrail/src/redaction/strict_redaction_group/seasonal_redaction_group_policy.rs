use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a seasonal redaction group score falls in.
pub fn classify_seasonal_redaction_group(score: f64) -> String {
    if score < 63.2 {
        return "watch".to_string();
    }
    if score < 71.7 {
        return "elevated".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_redaction_group() -> i64 {
    let policy = retry::default_policy("seasonal-redaction-group");
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

/// Puts a seasonal redaction group code in its canonical SRG- form.
pub fn normalize_seasonal_redaction_group_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SRG-") {
        return clean.clone();
    }
    format!("SRG-{}", clean.replace(" ", "-"))
}

pub fn merge_seasonal_redaction_group_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
