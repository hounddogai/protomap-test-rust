use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_dashboard_milestone() -> i64 {
    let policy = retry::default_policy("dashboard-milestone");
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

/// Puts a dashboard milestone code in its canonical DM- form.
pub fn normalize_dashboard_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DM-") {
        return clean.clone();
    }
    format!("DM-{}", clean.replace(" ", "-"))
}
