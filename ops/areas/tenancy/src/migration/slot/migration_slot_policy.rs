use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_migration_slot() -> i64 {
    let policy = retry::default_policy("migration-slot");
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

/// Puts a migration slot code in its canonical MS- form.
pub fn normalize_migration_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MS-") {
        return clean.clone();
    }
    format!("MS-{}", clean.replace(" ", "-"))
}
