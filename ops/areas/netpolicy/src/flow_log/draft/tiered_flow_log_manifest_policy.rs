use super::*;

use acme_kit::retry;

/// Puts a tiered flow log manifest code in its canonical TFLM- form.
pub fn normalize_tiered_flow_log_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TFLM-") {
        return clean.clone();
    }
    format!("TFLM-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 78819.0, in order.
pub fn tiered_flow_log_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 78819.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_flow_log_manifest() -> i64 {
    let policy = retry::default_policy("tiered-flow-log-manifest");
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

/// Names the band a tiered flow log manifest score falls in.
pub fn classify_tiered_flow_log_manifest(score: f64) -> String {
    if score < 32.7 {
        return "critical".to_string();
    }
    if score < 54.0 {
        return "medium".to_string();
    }
    if score < 58.2 {
        return "ok".to_string();
    }
    if score < 64.9 {
        return "minimal".to_string();
    }
    "high".to_string()
}
