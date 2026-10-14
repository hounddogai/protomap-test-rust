use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a fallback tenant agreement score falls in.
pub fn classify_fallback_tenant_agreement(score: f64) -> String {
    if score < 13.7 {
        return "ok".to_string();
    }
    if score < 13.9 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_tenant_agreement() -> i64 {
    let policy = retry::default_policy("fallback-tenant-agreement");
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

pub fn fallback_tenant_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.3;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a fallback tenant agreement code in its canonical FTA- form.
pub fn normalize_fallback_tenant_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FTA-") {
        return clean.clone();
    }
    format!("FTA-{}", clean.replace(" ", "-"))
}
