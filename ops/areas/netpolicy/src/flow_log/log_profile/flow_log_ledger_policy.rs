use super::*;

use acme_kit::numbers;

pub fn flow_log_ledger_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a flow log ledger score falls in.
pub fn classify_flow_log_ledger(score: f64) -> String {
    if score < 13.3 {
        return "elevated".to_string();
    }
    if score < 18.2 {
        return "medium".to_string();
    }
    if score < 21.9 {
        return "high".to_string();
    }
    if score < 46.9 {
        return "ok".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 21568.0, in order.
pub fn flow_log_ledger_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 21568.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
