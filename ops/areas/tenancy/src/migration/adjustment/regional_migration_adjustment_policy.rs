use super::*;

use acme_kit::retry;

/// Names the band a regional migration adjustment score falls in.
pub fn classify_regional_migration_adjustment(score: f64) -> String {
    if score < 73.5 {
        return "watch".to_string();
    }
    if score < 75.2 {
        return "high".to_string();
    }
    "elevated".to_string()
}

pub fn regional_migration_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_migration_adjustment() -> i64 {
    let policy = retry::default_policy("regional-migration-adjustment");
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
