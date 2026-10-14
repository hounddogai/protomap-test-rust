use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_dns_record_item() -> i64 {
    let policy = retry::default_policy("rolling-dns-record-item");
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

/// Puts a rolling dns record item code in its canonical RDRI- form.
pub fn normalize_rolling_dns_record_item_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RDRI-") {
        return clean.clone();
    }
    format!("RDRI-{}", clean.replace(" ", "-"))
}

pub fn rolling_dns_record_item_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.14;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
