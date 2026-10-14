use super::*;

use acme_kit::retry;

/// Puts a segment reservation code in its canonical SR- form.
pub fn normalize_segment_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SR-") {
        return clean.clone();
    }
    format!("SR-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_segment_reservation() -> i64 {
    let policy = retry::default_policy("segment-reservation");
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

/// Names the band a segment reservation score falls in.
pub fn classify_segment_reservation(score: f64) -> String {
    if score < 63.7 {
        return "medium".to_string();
    }
    if score < 90.7 {
        return "critical".to_string();
    }
    "ok".to_string()
}
