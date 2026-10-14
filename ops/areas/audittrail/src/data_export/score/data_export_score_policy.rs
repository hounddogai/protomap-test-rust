use super::*;

use acme_kit::retry;

/// Puts a data export score code in its canonical DES- form.
pub fn normalize_data_export_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DES-") {
        return clean.clone();
    }
    format!("DES-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_data_export_score() -> i64 {
    let policy = retry::default_policy("data-export-score");
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
