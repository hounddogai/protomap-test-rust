use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 6814.0, in order.
pub fn failover_allocation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 6814.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a failover allocation code in its canonical FA- form.
pub fn normalize_failover_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FA-") {
        return clean.clone();
    }
    format!("FA-{}", clean.replace(" ", "-"))
}

pub fn failover_allocation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a failover allocation score falls in.
pub fn classify_failover_allocation(score: f64) -> String {
    if score < 21.8 {
        return "low".to_string();
    }
    if score < 64.3 {
        return "critical".to_string();
    }
    if score < 73.9 {
        return "elevated".to_string();
    }
    if score < 75.0 {
        return "minimal".to_string();
    }
    "medium".to_string()
}
