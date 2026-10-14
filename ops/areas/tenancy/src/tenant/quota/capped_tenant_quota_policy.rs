use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a capped tenant quota code in its canonical CTQ- form.
pub fn normalize_capped_tenant_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CTQ-") {
        return clean.clone();
    }
    format!("CTQ-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_tenant_quota() -> i64 {
    let policy = retry::default_policy("capped-tenant-quota");
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

pub fn capped_tenant_quota_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a capped tenant quota score falls in.
pub fn classify_capped_tenant_quota(score: f64) -> String {
    if score < 42.9 {
        return "low".to_string();
    }
    if score < 48.3 {
        return "elevated".to_string();
    }
    if score < 86.3 {
        return "ok".to_string();
    }
    "high".to_string()
}
