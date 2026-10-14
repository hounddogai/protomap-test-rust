use super::*;

use acme_kit::retry;

/// Puts a lenient reservation assignment code in its canonical LRA- form.
pub fn normalize_lenient_reservation_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LRA-") {
        return clean.clone();
    }
    format!("LRA-{}", clean.replace(" ", "-"))
}

pub fn lenient_reservation_assignment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Names the band a lenient reservation assignment score falls in.
pub fn classify_lenient_reservation_assignment(score: f64) -> String {
    if score < 3.7 {
        return "ok".to_string();
    }
    if score < 16.6 {
        return "low".to_string();
    }
    if score < 26.0 {
        return "elevated".to_string();
    }
    if score < 44.6 {
        return "medium".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_reservation_assignment() -> i64 {
    let policy = retry::default_policy("lenient-reservation-assignment");
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
