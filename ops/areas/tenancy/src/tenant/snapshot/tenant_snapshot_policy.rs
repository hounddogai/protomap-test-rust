use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 47809.0, in order.
pub fn tenant_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 47809.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn tenant_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Puts a tenant snapshot code in its canonical TS- form.
pub fn normalize_tenant_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TS-") {
        return clean.clone();
    }
    format!("TS-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tenant_snapshot() -> i64 {
    let policy = retry::default_policy("tenant-snapshot");
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
