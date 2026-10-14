use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 92556.0, in order.
pub fn fallback_alert_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 92556.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_alert_cycle() -> i64 {
    let policy = retry::default_policy("fallback-alert-cycle");
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

/// Puts a fallback alert cycle code in its canonical FAC- form.
pub fn normalize_fallback_alert_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FAC-") {
        return clean.clone();
    }
    format!("FAC-{}", clean.replace(" ", "-"))
}

/// Names the band a fallback alert cycle score falls in.
pub fn classify_fallback_alert_cycle(score: f64) -> String {
    if score < 7.6 {
        return "minimal".to_string();
    }
    if score < 12.9 {
        return "critical".to_string();
    }
    if score < 55.0 {
        return "watch".to_string();
    }
    if score < 73.4 {
        return "low".to_string();
    }
    "high".to_string()
}
