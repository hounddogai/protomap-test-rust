use super::*;

use acme_kit::retry;

/// Puts a global evidence window code in its canonical GEW- form.
pub fn normalize_global_evidence_window_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GEW-") {
        return clean.clone();
    }
    format!("GEW-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_evidence_window() -> i64 {
    let policy = retry::default_policy("global-evidence-window");
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
