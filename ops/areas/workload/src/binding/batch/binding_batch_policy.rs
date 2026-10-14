use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_binding_batch() -> i64 {
    let policy = retry::default_policy("binding-batch");
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

/// Puts a binding batch code in its canonical BB- form.
pub fn normalize_binding_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BB-") {
        return clean.clone();
    }
    format!("BB-{}", clean.replace(" ", "-"))
}
