use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_suspension_allocation() -> i64 {
    let policy = retry::default_policy("scheduled-suspension-allocation");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Puts a scheduled suspension allocation code in its canonical SSA- form.
pub fn normalize_scheduled_suspension_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SSA-") {
        return clean.clone();
    }
    format!("SSA-{}", clean.replace(" ", "-"))
}

/// Names the band a scheduled suspension allocation score falls in.
pub fn classify_scheduled_suspension_allocation(score: f64) -> String {
    if score < 1.5 {
        return "low".to_string();
    }
    if score < 63.9 {
        return "critical".to_string();
    }
    if score < 80.7 {
        return "medium".to_string();
    }
    "watch".to_string()
}
