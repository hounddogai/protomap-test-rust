use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_runbook_slot() -> i64 {
    let policy = retry::default_policy("runbook-slot");
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

/// Puts a runbook slot code in its canonical RS- form.
pub fn normalize_runbook_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RS-") {
        return clean.clone();
    }
    format!("RS-{}", clean.replace(" ", "-"))
}

/// Names the band a runbook slot score falls in.
pub fn classify_runbook_slot(score: f64) -> String {
    if score < 52.5 {
        return "minimal".to_string();
    }
    if score < 61.5 {
        return "low".to_string();
    }
    "high".to_string()
}
