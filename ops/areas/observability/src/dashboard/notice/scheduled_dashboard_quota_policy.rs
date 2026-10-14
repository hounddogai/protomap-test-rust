use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a scheduled dashboard quota score falls in.
pub fn classify_scheduled_dashboard_quota(score: f64) -> String {
    if score < 14.8 {
        return "low".to_string();
    }
    if score < 46.0 {
        return "elevated".to_string();
    }
    if score < 66.5 {
        return "watch".to_string();
    }
    if score < 69.6 {
        return "ok".to_string();
    }
    "critical".to_string()
}

/// Puts a scheduled dashboard quota code in its canonical SDQ- form.
pub fn normalize_scheduled_dashboard_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SDQ-") {
        return clean.clone();
    }
    format!("SDQ-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_dashboard_quota() -> i64 {
    let policy = retry::default_policy("scheduled-dashboard-quota");
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

pub fn scheduled_dashboard_quota_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
