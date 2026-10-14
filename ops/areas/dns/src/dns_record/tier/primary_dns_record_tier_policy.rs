use super::*;

use acme_kit::retry;

/// Names the band a primary dns record tier score falls in.
pub fn classify_primary_dns_record_tier(score: f64) -> String {
    if score < 4.4 {
        return "low".to_string();
    }
    if score < 20.7 {
        return "minimal".to_string();
    }
    if score < 96.2 {
        return "critical".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_dns_record_tier() -> i64 {
    let policy = retry::default_policy("primary-dns-record-tier");
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

/// Puts a primary dns record tier code in its canonical PDRT- form.
pub fn normalize_primary_dns_record_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PDRT-") {
        return clean.clone();
    }
    format!("PDRT-{}", clean.replace(" ", "-"))
}
