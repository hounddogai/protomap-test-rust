use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a scheduled peering metric code in its canonical SPM- form.
pub fn normalize_scheduled_peering_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SPM-") {
        return clean.clone();
    }
    format!("SPM-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_peering_metric() -> i64 {
    let policy = retry::default_policy("scheduled-peering-metric");
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

pub fn scheduled_peering_metric_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.61;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a scheduled peering metric score falls in.
pub fn classify_scheduled_peering_metric(score: f64) -> String {
    if score < 58.2 {
        return "elevated".to_string();
    }
    if score < 64.5 {
        return "critical".to_string();
    }
    if score < 96.1 {
        return "ok".to_string();
    }
    "minimal".to_string()
}
