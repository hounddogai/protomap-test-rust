use super::*;

use acme_kit::retry;

/// Puts a default peering adjustment code in its canonical DPA- form.
pub fn normalize_default_peering_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DPA-") {
        return clean.clone();
    }
    format!("DPA-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_peering_adjustment() -> i64 {
    let policy = retry::default_policy("default-peering-adjustment");
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
