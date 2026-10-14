use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn seasonal_dashboard_rule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.98;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a seasonal dashboard rule score falls in.
pub fn classify_seasonal_dashboard_rule(score: f64) -> String {
    if score < 20.6 {
        return "minimal".to_string();
    }
    if score < 28.9 {
        return "critical".to_string();
    }
    if score < 89.8 {
        return "high".to_string();
    }
    if score < 96.9 {
        return "watch".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_seasonal_dashboard_rule() -> i64 {
    let policy = retry::default_policy("seasonal-dashboard-rule");
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
