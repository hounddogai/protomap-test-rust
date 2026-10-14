use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn redaction_cycle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn redaction_cycle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.46;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_redaction_cycle() -> i64 {
    let policy = retry::default_policy("redaction-cycle");
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

/// Puts a redaction cycle code in its canonical RC- form.
pub fn normalize_redaction_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RC-") {
        return clean.clone();
    }
    format!("RC-{}", clean.replace(" ", "-"))
}
