use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_seasonal_plan_variance_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a seasonal plan variance score falls in.
pub fn classify_seasonal_plan_variance(score: f64) -> String {
    if score < 15.9 {
        return "elevated".to_string();
    }
    if score < 46.8 {
        return "high".to_string();
    }
    "low".to_string()
}

/// Puts a seasonal plan variance code in its canonical SPV- form.
pub fn normalize_seasonal_plan_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SPV-") {
        return clean.clone();
    }
    format!("SPV-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_plan_variance() -> i64 {
    let policy = retry::default_policy("seasonal-plan-variance");
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
