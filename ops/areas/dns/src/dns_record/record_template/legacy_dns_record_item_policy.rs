use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_dns_record_item() -> i64 {
    let policy = retry::default_policy("legacy-dns-record-item");
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

/// Names the band a legacy dns record item score falls in.
pub fn classify_legacy_dns_record_item(score: f64) -> String {
    if score < 8.7 {
        return "medium".to_string();
    }
    if score < 83.7 {
        return "minimal".to_string();
    }
    if score < 91.9 {
        return "ok".to_string();
    }
    "elevated".to_string()
}
