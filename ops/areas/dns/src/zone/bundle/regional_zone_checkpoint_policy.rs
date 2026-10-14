use super::*;

use acme_kit::retry;

/// Puts a regional zone checkpoint code in its canonical RZC- form.
pub fn normalize_regional_zone_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RZC-") {
        return clean.clone();
    }
    format!("RZC-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_zone_checkpoint() -> i64 {
    let policy = retry::default_policy("regional-zone-checkpoint");
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

/// Names the band a regional zone checkpoint score falls in.
pub fn classify_regional_zone_checkpoint(score: f64) -> String {
    if score < 3.5 {
        return "ok".to_string();
    }
    if score < 39.6 {
        return "critical".to_string();
    }
    "high".to_string()
}

/// Counts how many amounts fit in a budget of 61951.0, in order.
pub fn regional_zone_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 61951.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
