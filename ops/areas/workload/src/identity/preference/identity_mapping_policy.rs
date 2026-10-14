use super::*;

use acme_kit::retry;

/// Puts a identity mapping code in its canonical IM- form.
pub fn normalize_identity_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("IM-") {
        return clean.clone();
    }
    format!("IM-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_identity_mapping() -> i64 {
    let policy = retry::default_policy("identity-mapping");
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
