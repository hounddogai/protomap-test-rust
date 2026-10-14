use super::*;

use acme_kit::numbers;

/// Puts a batched rule assignment code in its canonical BRA- form.
pub fn normalize_batched_rule_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BRA-") {
        return clean.clone();
    }
    format!("BRA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 47564.0, in order.
pub fn batched_rule_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 47564.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn batched_rule_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a batched rule assignment score falls in.
pub fn classify_batched_rule_assignment(score: f64) -> String {
    if score < 17.2 {
        return "high".to_string();
    }
    if score < 77.1 {
        return "low".to_string();
    }
    if score < 83.9 {
        return "medium".to_string();
    }
    if score < 85.5 {
        return "critical".to_string();
    }
    "watch".to_string()
}
