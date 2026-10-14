use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn default_data_export_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.29;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a default data export draft score falls in.
pub fn classify_default_data_export_draft(score: f64) -> String {
    if score < 1.8 {
        return "low".to_string();
    }
    if score < 6.2 {
        return "minimal".to_string();
    }
    if score < 41.4 {
        return "elevated".to_string();
    }
    if score < 71.4 {
        return "ok".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_data_export_draft() -> i64 {
    let policy = retry::default_policy("default-data-export-draft");
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
