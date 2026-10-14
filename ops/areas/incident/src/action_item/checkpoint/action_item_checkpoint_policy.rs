use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_action_item_checkpoint() -> i64 {
    let policy = retry::default_policy("action-item-checkpoint");
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

/// Puts a action item checkpoint code in its canonical AIC- form.
pub fn normalize_action_item_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AIC-") {
        return clean.clone();
    }
    format!("AIC-{}", clean.replace(" ", "-"))
}
