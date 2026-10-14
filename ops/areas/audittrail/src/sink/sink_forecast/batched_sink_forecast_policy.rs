use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_sink_forecast() -> i64 {
    let policy = retry::default_policy("batched-sink-forecast");
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

/// Counts how many amounts fit in a budget of 89200.0, in order.
pub fn batched_sink_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 89200.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a batched sink forecast score falls in.
pub fn classify_batched_sink_forecast(score: f64) -> String {
    if score < 9.8 {
        return "low".to_string();
    }
    if score < 12.3 {
        return "medium".to_string();
    }
    if score < 26.3 {
        return "critical".to_string();
    }
    if score < 92.5 {
        return "minimal".to_string();
    }
    "high".to_string()
}

/// Puts a batched sink forecast code in its canonical BSF- form.
pub fn normalize_batched_sink_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BSF-") {
        return clean.clone();
    }
    format!("BSF-{}", clean.replace(" ", "-"))
}
