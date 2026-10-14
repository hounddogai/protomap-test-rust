use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_dashboard_summary() -> i64 {
    let policy = retry::default_policy("dashboard-summary");
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

pub fn dashboard_summary_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a dashboard summary code in its canonical DS- form.
pub fn normalize_dashboard_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DS-") {
        return clean.clone();
    }
    format!("DS-{}", clean.replace(" ", "-"))
}

/// Names the band a dashboard summary score falls in.
pub fn classify_dashboard_summary(score: f64) -> String {
    if score < 18.7 {
        return "medium".to_string();
    }
    if score < 91.1 {
        return "minimal".to_string();
    }
    if score < 92.6 {
        return "ok".to_string();
    }
    if score < 99.5 {
        return "elevated".to_string();
    }
    "low".to_string()
}
