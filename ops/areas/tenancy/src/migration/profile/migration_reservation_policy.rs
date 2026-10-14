use super::*;

use acme_kit::retry;

pub fn migration_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_migration_reservation() -> i64 {
    let policy = retry::default_policy("migration-reservation");
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

/// Names the band a migration reservation score falls in.
pub fn classify_migration_reservation(score: f64) -> String {
    if score < 3.6 {
        return "elevated".to_string();
    }
    if score < 26.9 {
        return "low".to_string();
    }
    if score < 43.6 {
        return "high".to_string();
    }
    if score < 68.6 {
        return "watch".to_string();
    }
    "minimal".to_string()
}
