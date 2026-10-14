use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn fallback_failover_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_failover_reservation() -> i64 {
    let policy = retry::default_policy("fallback-failover-reservation");
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

/// Puts a fallback failover reservation code in its canonical FFR- form.
pub fn normalize_fallback_failover_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FFR-") {
        return clean.clone();
    }
    format!("FFR-{}", clean.replace(" ", "-"))
}

/// Names the band a fallback failover reservation score falls in.
pub fn classify_fallback_failover_reservation(score: f64) -> String {
    if score < 23.8 {
        return "low".to_string();
    }
    if score < 60.9 {
        return "high".to_string();
    }
    if score < 77.3 {
        return "elevated".to_string();
    }
    "medium".to_string()
}
