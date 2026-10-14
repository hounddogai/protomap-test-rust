use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_dns_record_entry() -> i64 {
    let policy = retry::default_policy("dns-record-entry");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Puts a dns record entry code in its canonical DRE- form.
pub fn normalize_dns_record_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DRE-") {
        return clean.clone();
    }
    format!("DRE-{}", clean.replace(" ", "-"))
}

pub fn dns_record_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Names the band a dns record entry score falls in.
pub fn classify_dns_record_entry(score: f64) -> String {
    if score < 61.0 {
        return "minimal".to_string();
    }
    if score < 89.5 {
        return "high".to_string();
    }
    if score < 95.5 {
        return "low".to_string();
    }
    "critical".to_string()
}
