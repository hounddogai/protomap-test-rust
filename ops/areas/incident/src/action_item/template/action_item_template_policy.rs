use super::*;

use acme_kit::retry;

/// Puts a action item template code in its canonical AIT- form.
pub fn normalize_action_item_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AIT-") {
        return clean.clone();
    }
    format!("AIT-{}", clean.replace(" ", "-"))
}

pub fn action_item_template_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_action_item_template() -> i64 {
    let policy = retry::default_policy("action-item-template");
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
