use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_page_proposal() -> i64 {
    let policy = retry::default_policy("adaptive-page-proposal");
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

/// Names the band a adaptive page proposal score falls in.
pub fn classify_adaptive_page_proposal(score: f64) -> String {
    if score < 36.7 {
        return "watch".to_string();
    }
    if score < 50.3 {
        return "high".to_string();
    }
    if score < 87.6 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

pub fn adaptive_page_proposal_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.53;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_adaptive_page_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
