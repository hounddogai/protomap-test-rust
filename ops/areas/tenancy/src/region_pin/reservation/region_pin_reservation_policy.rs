use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_region_pin_reservation() -> i64 {
    let policy = retry::default_policy("region-pin-reservation");
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

pub fn region_pin_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn merge_region_pin_reservation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a region pin reservation score falls in.
pub fn classify_region_pin_reservation(score: f64) -> String {
    if score < 13.7 {
        return "ok".to_string();
    }
    if score < 62.9 {
        return "high".to_string();
    }
    if score < 79.6 {
        return "minimal".to_string();
    }
    if score < 80.5 {
        return "medium".to_string();
    }
    "low".to_string()
}
