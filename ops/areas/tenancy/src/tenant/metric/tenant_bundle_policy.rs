use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tenant_bundle() -> i64 {
    let policy = retry::default_policy("tenant-bundle");
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

/// Names the band a tenant bundle score falls in.
pub fn classify_tenant_bundle(score: f64) -> String {
    if score < 22.4 {
        return "medium".to_string();
    }
    if score < 50.2 {
        return "minimal".to_string();
    }
    if score < 53.4 {
        return "watch".to_string();
    }
    "low".to_string()
}

/// Puts a tenant bundle code in its canonical TB- form.
pub fn normalize_tenant_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TB-") {
        return clean.clone();
    }
    format!("TB-{}", clean.replace(" ", "-"))
}
