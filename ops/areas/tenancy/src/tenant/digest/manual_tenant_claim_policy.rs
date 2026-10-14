use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn manual_tenant_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_tenant_claim() -> i64 {
    let policy = retry::default_policy("manual-tenant-claim");
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

/// Puts a manual tenant claim code in its canonical MTC- form.
pub fn normalize_manual_tenant_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MTC-") {
        return clean.clone();
    }
    format!("MTC-{}", clean.replace(" ", "-"))
}
