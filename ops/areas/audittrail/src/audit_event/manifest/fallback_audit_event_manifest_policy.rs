use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn fallback_audit_event_manifest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.32;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a fallback audit event manifest code in its canonical FAEM- form.
pub fn normalize_fallback_audit_event_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FAEM-") {
        return clean.clone();
    }
    format!("FAEM-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_audit_event_manifest() -> i64 {
    let policy = retry::default_policy("fallback-audit-event-manifest");
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
