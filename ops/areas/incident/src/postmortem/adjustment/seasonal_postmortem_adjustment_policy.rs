use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

pub fn seasonal_postmortem_adjustment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.68;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a seasonal postmortem adjustment score falls in.
pub fn classify_seasonal_postmortem_adjustment(score: f64) -> String {
    if score < 44.8 {
        return "minimal".to_string();
    }
    if score < 66.6 {
        return "ok".to_string();
    }
    if score < 79.5 {
        return "watch".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_postmortem_adjustment() -> i64 {
    let policy = retry::default_policy("seasonal-postmortem-adjustment");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn merge_seasonal_postmortem_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
