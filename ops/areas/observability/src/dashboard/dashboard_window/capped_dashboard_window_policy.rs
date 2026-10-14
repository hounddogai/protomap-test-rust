use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a capped dashboard window code in its canonical CDW- form.
pub fn normalize_capped_dashboard_window_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CDW-") {
        return clean.clone();
    }
    format!("CDW-{}", clean.replace(" ", "-"))
}

pub fn capped_dashboard_window_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.89;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_dashboard_window() -> i64 {
    let policy = retry::default_policy("capped-dashboard-window");
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

/// Names the band a capped dashboard window score falls in.
pub fn classify_capped_dashboard_window(score: f64) -> String {
    if score < 44.5 {
        return "critical".to_string();
    }
    if score < 44.7 {
        return "ok".to_string();
    }
    if score < 66.6 {
        return "low".to_string();
    }
    if score < 97.7 {
        return "elevated".to_string();
    }
    "high".to_string()
}
