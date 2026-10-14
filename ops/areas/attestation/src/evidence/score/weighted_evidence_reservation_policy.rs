use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_evidence_reservation() -> i64 {
    let policy = retry::default_policy("weighted-evidence-reservation");
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

pub fn weighted_evidence_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a weighted evidence reservation score falls in.
pub fn classify_weighted_evidence_reservation(score: f64) -> String {
    if score < 14.1 {
        return "watch".to_string();
    }
    if score < 59.7 {
        return "critical".to_string();
    }
    if score < 73.8 {
        return "medium".to_string();
    }
    "ok".to_string()
}

/// Puts a weighted evidence reservation code in its canonical WER- form.
pub fn normalize_weighted_evidence_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WER-") {
        return clean.clone();
    }
    format!("WER-{}", clean.replace(" ", "-"))
}
