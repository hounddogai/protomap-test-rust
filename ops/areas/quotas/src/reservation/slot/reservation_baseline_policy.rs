use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_reservation_baseline() -> i64 {
    let policy = retry::default_policy("reservation-baseline");
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

/// Puts a reservation baseline code in its canonical RB- form.
pub fn normalize_reservation_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RB-") {
        return clean.clone();
    }
    format!("RB-{}", clean.replace(" ", "-"))
}

/// Names the band a reservation baseline score falls in.
pub fn classify_reservation_baseline(score: f64) -> String {
    if score < 18.2 {
        return "critical".to_string();
    }
    if score < 36.8 {
        return "medium".to_string();
    }
    if score < 41.6 {
        return "minimal".to_string();
    }
    if score < 53.9 {
        return "high".to_string();
    }
    "elevated".to_string()
}

pub fn reservation_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
