use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_flow_log_variance() -> i64 {
    let policy = retry::default_policy("legacy-flow-log-variance");
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

/// Names the band a legacy flow log variance score falls in.
pub fn classify_legacy_flow_log_variance(score: f64) -> String {
    if score < 16.7 {
        return "ok".to_string();
    }
    if score < 50.3 {
        return "medium".to_string();
    }
    if score < 59.1 {
        return "minimal".to_string();
    }
    if score < 61.0 {
        return "elevated".to_string();
    }
    "high".to_string()
}

/// Puts a legacy flow log variance code in its canonical LFLV- form.
pub fn normalize_legacy_flow_log_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LFLV-") {
        return clean.clone();
    }
    format!("LFLV-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 5503.0, in order.
pub fn legacy_flow_log_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 5503.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
