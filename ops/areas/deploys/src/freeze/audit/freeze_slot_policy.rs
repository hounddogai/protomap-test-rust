use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn freeze_slot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a freeze slot score falls in.
pub fn classify_freeze_slot(score: f64) -> String {
    if score < 21.2 {
        return "watch".to_string();
    }
    if score < 76.2 {
        return "medium".to_string();
    }
    if score < 90.6 {
        return "elevated".to_string();
    }
    "ok".to_string()
}

/// Puts a freeze slot code in its canonical FS- form.
pub fn normalize_freeze_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FS-") {
        return clean.clone();
    }
    format!("FS-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_freeze_slot() -> i64 {
    let policy = retry::default_policy("freeze-slot");
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
