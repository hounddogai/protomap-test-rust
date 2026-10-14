use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn lenient_dns_record_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a lenient dns record baseline score falls in.
pub fn classify_lenient_dns_record_baseline(score: f64) -> String {
    if score < 14.4 {
        return "low".to_string();
    }
    if score < 34.3 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

pub fn lenient_dns_record_baseline_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.8;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_dns_record_baseline() -> i64 {
    let policy = retry::default_policy("lenient-dns-record-baseline");
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
