use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_dashboard_agreement() -> i64 {
    let policy = retry::default_policy("dashboard-agreement");
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

/// Puts a dashboard agreement code in its canonical DA- form.
pub fn normalize_dashboard_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DA-") {
        return clean.clone();
    }
    format!("DA-{}", clean.replace(" ", "-"))
}
