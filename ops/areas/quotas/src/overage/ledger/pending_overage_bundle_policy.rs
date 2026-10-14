use super::*;

use acme_kit::retry;

/// Puts a pending overage bundle code in its canonical POB- form.
pub fn normalize_pending_overage_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("POB-") {
        return clean.clone();
    }
    format!("POB-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_overage_bundle() -> i64 {
    let policy = retry::default_policy("pending-overage-bundle");
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
