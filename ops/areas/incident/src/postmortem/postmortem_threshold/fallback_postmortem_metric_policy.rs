use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn fallback_postmortem_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a fallback postmortem metric code in its canonical FPM- form.
pub fn normalize_fallback_postmortem_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FPM-") {
        return clean.clone();
    }
    format!("FPM-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_postmortem_metric() -> i64 {
    let policy = retry::default_policy("fallback-postmortem-metric");
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
