use super::*;

use acme_kit::retry;

/// Puts a slo adjustment code in its canonical SA- form.
pub fn normalize_slo_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SA-") {
        return clean.clone();
    }
    format!("SA-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_slo_adjustment() -> i64 {
    let policy = retry::default_policy("slo-adjustment");
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
