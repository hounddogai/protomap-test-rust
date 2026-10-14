use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a action item cycle score falls in.
pub fn classify_action_item_cycle(score: f64) -> String {
    if score < 25.7 {
        return "low".to_string();
    }
    if score < 40.6 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_action_item_cycle() -> i64 {
    let policy = retry::default_policy("action-item-cycle");
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

/// Puts a action item cycle code in its canonical AIC- form.
pub fn normalize_action_item_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AIC-") {
        return clean.clone();
    }
    format!("AIC-{}", clean.replace(" ", "-"))
}

pub fn action_item_cycle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.98;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
