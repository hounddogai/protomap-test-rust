use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a sink entry code in its canonical SE- form.
pub fn normalize_sink_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SE-") {
        return clean.clone();
    }
    format!("SE-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_sink_entry() -> i64 {
    let policy = retry::default_policy("sink-entry");
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

pub fn sink_entry_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
