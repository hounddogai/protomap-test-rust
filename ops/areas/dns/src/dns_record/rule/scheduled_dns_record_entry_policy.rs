use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_dns_record_entry() -> i64 {
    let policy = retry::default_policy("scheduled-dns-record-entry");
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

/// Puts a scheduled dns record entry code in its canonical SDRE- form.
pub fn normalize_scheduled_dns_record_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SDRE-") {
        return clean.clone();
    }
    format!("SDRE-{}", clean.replace(" ", "-"))
}

pub fn scheduled_dns_record_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 22729.0, in order.
pub fn scheduled_dns_record_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 22729.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
