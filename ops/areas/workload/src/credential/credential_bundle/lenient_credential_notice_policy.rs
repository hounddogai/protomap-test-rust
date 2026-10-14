use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn lenient_credential_notice_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn lenient_credential_notice_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.85;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_credential_notice() -> i64 {
    let policy = retry::default_policy("lenient-credential-notice");
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

/// Puts a lenient credential notice code in its canonical LCN- form.
pub fn normalize_lenient_credential_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LCN-") {
        return clean.clone();
    }
    format!("LCN-{}", clean.replace(" ", "-"))
}
