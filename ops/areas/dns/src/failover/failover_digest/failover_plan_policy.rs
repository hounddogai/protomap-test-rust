use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 21342.0, in order.
pub fn failover_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 21342.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn failover_plan_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a failover plan score falls in.
pub fn classify_failover_plan(score: f64) -> String {
    if score < 7.3 {
        return "minimal".to_string();
    }
    if score < 79.2 {
        return "medium".to_string();
    }
    if score < 84.8 {
        return "ok".to_string();
    }
    if score < 99.2 {
        return "high".to_string();
    }
    "critical".to_string()
}
