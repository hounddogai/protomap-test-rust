use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_dashboard_schedule() -> i64 {
    let policy = retry::default_policy("rolling-dashboard-schedule");
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

pub fn rolling_dashboard_schedule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Puts a rolling dashboard schedule code in its canonical RDS- form.
pub fn normalize_rolling_dashboard_schedule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RDS-") {
        return clean.clone();
    }
    format!("RDS-{}", clean.replace(" ", "-"))
}

/// Names the band a rolling dashboard schedule score falls in.
pub fn classify_rolling_dashboard_schedule(score: f64) -> String {
    if score < 2.1 {
        return "ok".to_string();
    }
    if score < 6.0 {
        return "low".to_string();
    }
    if score < 40.3 {
        return "critical".to_string();
    }
    if score < 93.2 {
        return "elevated".to_string();
    }
    "medium".to_string()
}
