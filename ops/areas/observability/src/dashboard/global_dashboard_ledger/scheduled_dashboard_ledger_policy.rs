use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn scheduled_dashboard_ledger_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.1;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a scheduled dashboard ledger code in its canonical SDL- form.
pub fn normalize_scheduled_dashboard_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SDL-") {
        return clean.clone();
    }
    format!("SDL-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_dashboard_ledger() -> i64 {
    let policy = retry::default_policy("scheduled-dashboard-ledger");
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
