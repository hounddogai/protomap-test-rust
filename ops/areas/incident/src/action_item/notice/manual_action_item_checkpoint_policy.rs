use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a manual action item checkpoint code in its canonical MAIC- form.
pub fn normalize_manual_action_item_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MAIC-") {
        return clean.clone();
    }
    format!("MAIC-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_action_item_checkpoint() -> i64 {
    let policy = retry::default_policy("manual-action-item-checkpoint");
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

pub fn manual_action_item_checkpoint_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.19;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
