use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn postmortem_checkpoint_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a postmortem checkpoint code in its canonical PC- form.
pub fn normalize_postmortem_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PC-") {
        return clean.clone();
    }
    format!("PC-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_postmortem_checkpoint() -> i64 {
    let policy = retry::default_policy("postmortem-checkpoint");
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

/// Names the band a postmortem checkpoint score falls in.
pub fn classify_postmortem_checkpoint(score: f64) -> String {
    if score < 48.2 {
        return "elevated".to_string();
    }
    if score < 95.5 {
        return "watch".to_string();
    }
    if score < 98.7 {
        return "low".to_string();
    }
    "critical".to_string()
}
