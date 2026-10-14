use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_alert_manifest() -> i64 {
    let policy = retry::default_policy("alert-manifest");
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

/// Names the band a alert manifest score falls in.
pub fn classify_alert_manifest(score: f64) -> String {
    if score < 8.7 {
        return "ok".to_string();
    }
    if score < 90.5 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 9051.0, in order.
pub fn alert_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 9051.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a alert manifest code in its canonical AM- form.
pub fn normalize_alert_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AM-") {
        return clean.clone();
    }
    format!("AM-{}", clean.replace(" ", "-"))
}
