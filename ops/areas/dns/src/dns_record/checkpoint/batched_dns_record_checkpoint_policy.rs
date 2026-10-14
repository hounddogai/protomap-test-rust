use super::*;

use acme_kit::retry;

/// Puts a batched dns record checkpoint code in its canonical BDRC- form.
pub fn normalize_batched_dns_record_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BDRC-") {
        return clean.clone();
    }
    format!("BDRC-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_dns_record_checkpoint() -> i64 {
    let policy = retry::default_policy("batched-dns-record-checkpoint");
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
