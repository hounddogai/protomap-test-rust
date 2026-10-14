use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn zone_plan_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_zone_plan() -> i64 {
    let policy = retry::default_policy("zone-plan");
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

/// Puts a zone plan code in its canonical ZP- form.
pub fn normalize_zone_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ZP-") {
        return clean.clone();
    }
    format!("ZP-{}", clean.replace(" ", "-"))
}
