use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a trust domain profile score falls in.
pub fn classify_trust_domain_profile(score: f64) -> String {
    if score < 17.2 {
        return "high".to_string();
    }
    if score < 55.4 {
        return "medium".to_string();
    }
    if score < 72.4 {
        return "low".to_string();
    }
    if score < 80.8 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_trust_domain_profile() -> i64 {
    let policy = retry::default_policy("trust-domain-profile");
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

pub fn trust_domain_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn merge_trust_domain_profile_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
