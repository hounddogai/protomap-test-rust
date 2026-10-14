use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn data_export_cycle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.52;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_data_export_cycle() -> i64 {
    let policy = retry::default_policy("data-export-cycle");
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

pub fn data_export_cycle_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Names the band a data export cycle score falls in.
pub fn classify_data_export_cycle(score: f64) -> String {
    if score < 6.1 {
        return "ok".to_string();
    }
    if score < 60.8 {
        return "minimal".to_string();
    }
    if score < 87.1 {
        return "critical".to_string();
    }
    if score < 93.1 {
        return "medium".to_string();
    }
    "watch".to_string()
}
