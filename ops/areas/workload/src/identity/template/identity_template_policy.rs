use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_identity_template() -> i64 {
    let policy = retry::default_policy("identity-template");
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

/// Puts a identity template code in its canonical IT- form.
pub fn normalize_identity_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("IT-") {
        return clean.clone();
    }
    format!("IT-{}", clean.replace(" ", "-"))
}
