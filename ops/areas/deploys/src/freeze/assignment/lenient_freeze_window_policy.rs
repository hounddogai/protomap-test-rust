use super::*;

use acme_kit::retry;

/// Puts a lenient freeze window code in its canonical LFW- form.
pub fn normalize_lenient_freeze_window_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LFW-") {
        return clean.clone();
    }
    format!("LFW-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_freeze_window() -> i64 {
    let policy = retry::default_policy("lenient-freeze-window");
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
