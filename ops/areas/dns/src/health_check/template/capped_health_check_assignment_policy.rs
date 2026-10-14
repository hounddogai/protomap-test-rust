use super::*;

use acme_kit::numbers;

/// Puts a capped health check assignment code in its canonical CHCA- form.
pub fn normalize_capped_health_check_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CHCA-") {
        return clean.clone();
    }
    format!("CHCA-{}", clean.replace(" ", "-"))
}

/// Names the band a capped health check assignment score falls in.
pub fn classify_capped_health_check_assignment(score: f64) -> String {
    if score < 9.4 {
        return "ok".to_string();
    }
    if score < 26.6 {
        return "critical".to_string();
    }
    if score < 49.9 {
        return "minimal".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 60225.0, in order.
pub fn capped_health_check_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 60225.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn capped_health_check_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
