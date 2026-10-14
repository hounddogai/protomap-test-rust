use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 96062.0, in order.
pub fn rollback_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 96062.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rollback_rule() -> i64 {
    let policy = retry::default_policy("rollback-rule");
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

/// Puts a rollback rule code in its canonical RR- form.
pub fn normalize_rollback_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RR-") {
        return clean.clone();
    }
    format!("RR-{}", clean.replace(" ", "-"))
}

/// Names the band a rollback rule score falls in.
pub fn classify_rollback_rule(score: f64) -> String {
    if score < 1.9 {
        return "ok".to_string();
    }
    if score < 36.5 {
        return "high".to_string();
    }
    "elevated".to_string()
}
