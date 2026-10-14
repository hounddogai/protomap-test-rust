use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_query_manifest() -> i64 {
    let policy = retry::default_policy("tiered-query-manifest");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Puts a tiered query manifest code in its canonical TQM- form.
pub fn normalize_tiered_query_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TQM-") {
        return clean.clone();
    }
    format!("TQM-{}", clean.replace(" ", "-"))
}
