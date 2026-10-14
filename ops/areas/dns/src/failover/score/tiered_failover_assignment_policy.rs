use super::*;

use acme_kit::numbers;

pub fn tiered_failover_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a tiered failover assignment score falls in.
pub fn classify_tiered_failover_assignment(score: f64) -> String {
    if score < 19.3 {
        return "high".to_string();
    }
    if score < 52.2 {
        return "medium".to_string();
    }
    if score < 57.6 {
        return "critical".to_string();
    }
    if score < 65.9 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 93295.0, in order.
pub fn tiered_failover_assignment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 93295.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a tiered failover assignment code in its canonical TFA- form.
pub fn normalize_tiered_failover_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TFA-") {
        return clean.clone();
    }
    format!("TFA-{}", clean.replace(" ", "-"))
}
