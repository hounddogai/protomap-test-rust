use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_trust_domain_manifest() -> i64 {
    let policy = retry::default_policy("rolling-trust-domain-manifest");
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

/// Puts a rolling trust domain manifest code in its canonical RTDM- form.
pub fn normalize_rolling_trust_domain_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RTDM-") {
        return clean.clone();
    }
    format!("RTDM-{}", clean.replace(" ", "-"))
}
