use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a tiered tenant entry code in its canonical TTE- form.
pub fn normalize_tiered_tenant_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TTE-") {
        return clean.clone();
    }
    format!("TTE-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_tenant_entry() -> i64 {
    let policy = retry::default_policy("tiered-tenant-entry");
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

pub fn tiered_tenant_entry_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
