use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn strict_data_export_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_data_export_metric() -> i64 {
    let policy = retry::default_policy("strict-data-export-metric");
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

/// Names the band a strict data export metric score falls in.
pub fn classify_strict_data_export_metric(score: f64) -> String {
    if score < 6.2 {
        return "medium".to_string();
    }
    if score < 6.8 {
        return "critical".to_string();
    }
    if score < 21.1 {
        return "low".to_string();
    }
    if score < 49.2 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

pub fn strict_data_export_metric_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.94;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
