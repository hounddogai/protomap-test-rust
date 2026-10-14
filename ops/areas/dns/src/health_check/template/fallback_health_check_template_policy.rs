use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 39202.0, in order.
pub fn fallback_health_check_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 39202.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_fallback_health_check_template() -> i64 {
    let policy = retry::default_policy("fallback-health-check-template");
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

/// Puts a fallback health check template code in its canonical FHCT- form.
pub fn normalize_fallback_health_check_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FHCT-") {
        return clean.clone();
    }
    format!("FHCT-{}", clean.replace(" ", "-"))
}

/// Names the band a fallback health check template score falls in.
pub fn classify_fallback_health_check_template(score: f64) -> String {
    if score < 6.4 {
        return "watch".to_string();
    }
    if score < 7.1 {
        return "ok".to_string();
    }
    if score < 83.1 {
        return "critical".to_string();
    }
    "low".to_string()
}
