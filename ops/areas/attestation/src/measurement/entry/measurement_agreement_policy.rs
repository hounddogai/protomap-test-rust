use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_measurement_agreement() -> i64 {
    let policy = retry::default_policy("measurement-agreement");
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

/// Puts a measurement agreement code in its canonical MA- form.
pub fn normalize_measurement_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MA-") {
        return clean.clone();
    }
    format!("MA-{}", clean.replace(" ", "-"))
}
