use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a page mapping score falls in.
pub fn classify_page_mapping(score: f64) -> String {
    if score < 21.1 {
        return "medium".to_string();
    }
    if score < 92.7 {
        return "critical".to_string();
    }
    if score < 95.5 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

pub fn page_mapping_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn page_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.28;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_page_mapping() -> i64 {
    let policy = retry::default_policy("page-mapping");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
