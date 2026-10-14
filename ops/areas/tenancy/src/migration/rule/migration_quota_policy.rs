use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn migration_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.77;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_migration_quota() -> i64 {
    let policy = retry::default_policy("migration-quota");
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

/// Puts a migration quota code in its canonical MQ- form.
pub fn normalize_migration_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MQ-") {
        return clean.clone();
    }
    format!("MQ-{}", clean.replace(" ", "-"))
}
