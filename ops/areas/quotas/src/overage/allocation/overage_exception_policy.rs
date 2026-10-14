use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_overage_exception() -> i64 {
    let policy = retry::default_policy("overage-exception");
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

/// Puts a overage exception code in its canonical OE- form.
pub fn normalize_overage_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("OE-") {
        return clean.clone();
    }
    format!("OE-{}", clean.replace(" ", "-"))
}
