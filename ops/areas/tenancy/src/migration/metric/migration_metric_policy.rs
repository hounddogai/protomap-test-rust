use super::*;

use acme_kit::retry;

/// Puts a migration metric code in its canonical MM- form.
pub fn normalize_migration_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MM-") {
        return clean.clone();
    }
    format!("MM-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_migration_metric() -> i64 {
    let policy = retry::default_policy("migration-metric");
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
