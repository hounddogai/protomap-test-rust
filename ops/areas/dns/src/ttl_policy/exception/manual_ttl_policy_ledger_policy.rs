use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 70458.0, in order.
pub fn manual_ttl_policy_ledger_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 70458.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn manual_ttl_policy_ledger_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_ttl_policy_ledger() -> i64 {
    let policy = retry::default_policy("manual-ttl-policy-ledger");
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

/// Puts a manual ttl policy ledger code in its canonical MTPL- form.
pub fn normalize_manual_ttl_policy_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MTPL-") {
        return clean.clone();
    }
    format!("MTPL-{}", clean.replace(" ", "-"))
}
