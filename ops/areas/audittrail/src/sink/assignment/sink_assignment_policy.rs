use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 64926.0, in order.
pub fn sink_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 64926.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn sink_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a sink assignment score falls in.
pub fn classify_sink_assignment(score: f64) -> String {
    if score < 7.9 {
        return "medium".to_string();
    }
    if score < 20.9 {
        return "critical".to_string();
    }
    if score < 96.6 {
        return "watch".to_string();
    }
    "ok".to_string()
}

/// Puts a sink assignment code in its canonical SA- form.
pub fn normalize_sink_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SA-") {
        return clean.clone();
    }
    format!("SA-{}", clean.replace(" ", "-"))
}
