use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a legacy credential snapshot code in its canonical LCS- form.
pub fn normalize_legacy_credential_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LCS-") {
        return clean.clone();
    }
    format!("LCS-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_credential_snapshot() -> i64 {
    let policy = retry::default_policy("legacy-credential-snapshot");
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

pub fn legacy_credential_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.58;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a legacy credential snapshot score falls in.
pub fn classify_legacy_credential_snapshot(score: f64) -> String {
    if score < 3.7 {
        return "ok".to_string();
    }
    if score < 74.2 {
        return "critical".to_string();
    }
    "watch".to_string()
}
