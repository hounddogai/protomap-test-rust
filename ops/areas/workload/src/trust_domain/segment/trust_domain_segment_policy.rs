use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_trust_domain_segment() -> i64 {
    let policy = retry::default_policy("trust-domain-segment");
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

/// Puts a trust domain segment code in its canonical TDS- form.
pub fn normalize_trust_domain_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TDS-") {
        return clean.clone();
    }
    format!("TDS-{}", clean.replace(" ", "-"))
}
