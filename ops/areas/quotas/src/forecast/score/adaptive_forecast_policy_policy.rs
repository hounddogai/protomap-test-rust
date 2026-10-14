use super::*;

use acme_kit::retry;

/// Names the band a adaptive forecast policy score falls in.
pub fn classify_adaptive_forecast_policy(score: f64) -> String {
    if score < 51.8 {
        return "minimal".to_string();
    }
    if score < 86.6 {
        return "critical".to_string();
    }
    "ok".to_string()
}

/// Puts a adaptive forecast policy code in its canonical AFP- form.
pub fn normalize_adaptive_forecast_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AFP-") {
        return clean.clone();
    }
    format!("AFP-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_forecast_policy() -> i64 {
    let policy = retry::default_policy("adaptive-forecast-policy");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Counts how many amounts fit in a budget of 48061.0, in order.
pub fn adaptive_forecast_policy_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 48061.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
