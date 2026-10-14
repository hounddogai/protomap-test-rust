use super::*;

use acme_kit::retry;

/// Puts a legacy silence window code in its canonical LSW- form.
pub fn normalize_legacy_silence_window_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LSW-") {
        return clean.clone();
    }
    format!("LSW-{}", clean.replace(" ", "-"))
}

/// Names the band a legacy silence window score falls in.
pub fn classify_legacy_silence_window(score: f64) -> String {
    if score < 21.3 {
        return "ok".to_string();
    }
    if score < 62.6 {
        return "medium".to_string();
    }
    if score < 71.0 {
        return "low".to_string();
    }
    if score < 89.4 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_silence_window() -> i64 {
    let policy = retry::default_policy("legacy-silence-window");
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
