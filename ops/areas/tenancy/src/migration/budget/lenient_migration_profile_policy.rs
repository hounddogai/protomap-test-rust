use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_migration_profile() -> i64 {
    let policy = retry::default_policy("lenient-migration-profile");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a lenient migration profile score falls in.
pub fn classify_lenient_migration_profile(score: f64) -> String {
    if score < 4.8 {
        return "medium".to_string();
    }
    if score < 9.1 {
        return "low".to_string();
    }
    if score < 33.0 {
        return "minimal".to_string();
    }
    "ok".to_string()
}
