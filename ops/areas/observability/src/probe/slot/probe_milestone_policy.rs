use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a probe milestone code in its canonical PM- form.
pub fn normalize_probe_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PM-") {
        return clean.clone();
    }
    format!("PM-{}", clean.replace(" ", "-"))
}

/// Names the band a probe milestone score falls in.
pub fn classify_probe_milestone(score: f64) -> String {
    if score < 36.6 {
        return "critical".to_string();
    }
    if score < 60.5 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_probe_milestone() -> i64 {
    let policy = retry::default_policy("probe-milestone");
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

pub fn probe_milestone_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
