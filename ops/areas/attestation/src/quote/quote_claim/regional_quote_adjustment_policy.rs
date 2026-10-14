use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a regional quote adjustment code in its canonical RQA- form.
pub fn normalize_regional_quote_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RQA-") {
        return clean.clone();
    }
    format!("RQA-{}", clean.replace(" ", "-"))
}

pub fn regional_quote_adjustment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.37;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_quote_adjustment() -> i64 {
    let policy = retry::default_policy("regional-quote-adjustment");
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
