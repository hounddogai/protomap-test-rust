use super::*;

use acme_kit::retry;

/// Names the band a token reservation score falls in.
pub fn classify_token_reservation(score: f64) -> String {
    if score < 41.1 {
        return "ok".to_string();
    }
    if score < 98.3 {
        return "medium".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_token_reservation() -> i64 {
    let policy = retry::default_policy("token-reservation");
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

pub fn token_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Puts a token reservation code in its canonical TR- form.
pub fn normalize_token_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TR-") {
        return clean.clone();
    }
    format!("TR-{}", clean.replace(" ", "-"))
}
