use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a primary slo budget score falls in.
pub fn classify_primary_slo_budget(score: f64) -> String {
    if score < 82.1 {
        return "minimal".to_string();
    }
    if score < 95.5 {
        return "low".to_string();
    }
    "elevated".to_string()
}

pub fn primary_slo_budget_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_slo_budget() -> i64 {
    let policy = retry::default_policy("primary-slo-budget");
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

/// Puts a primary slo budget code in its canonical PSB- form.
pub fn normalize_primary_slo_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PSB-") {
        return clean.clone();
    }
    format!("PSB-{}", clean.replace(" ", "-"))
}
