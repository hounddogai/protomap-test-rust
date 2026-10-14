use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a scheduled dashboard rule code in its canonical SDR- form.
pub fn normalize_scheduled_dashboard_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SDR-") {
        return clean.clone();
    }
    format!("SDR-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_dashboard_rule() -> i64 {
    let policy = retry::default_policy("scheduled-dashboard-rule");
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

pub fn scheduled_dashboard_rule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.36;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
