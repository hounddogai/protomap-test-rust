use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a global identity profile score falls in.
pub fn classify_global_identity_profile(score: f64) -> String {
    if score < 22.2 {
        return "high".to_string();
    }
    if score < 27.6 {
        return "critical".to_string();
    }
    if score < 52.9 {
        return "watch".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_identity_profile() -> i64 {
    let policy = retry::default_policy("global-identity-profile");
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

pub fn merge_global_identity_profile_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
