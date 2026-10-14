use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn pending_migration_forecast_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.44;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a pending migration forecast score falls in.
pub fn classify_pending_migration_forecast(score: f64) -> String {
    if score < 36.2 {
        return "low".to_string();
    }
    if score < 51.2 {
        return "ok".to_string();
    }
    if score < 52.4 {
        return "elevated".to_string();
    }
    if score < 94.9 {
        return "high".to_string();
    }
    "minimal".to_string()
}

/// Puts a pending migration forecast code in its canonical PMF- form.
pub fn normalize_pending_migration_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PMF-") {
        return clean.clone();
    }
    format!("PMF-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_migration_forecast() -> i64 {
    let policy = retry::default_policy("pending-migration-forecast");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
