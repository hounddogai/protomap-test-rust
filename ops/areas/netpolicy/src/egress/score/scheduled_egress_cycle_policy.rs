use super::*;

/// Counts how many amounts fit in a budget of 9639.0, in order.
pub fn scheduled_egress_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 9639.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a scheduled egress cycle score falls in.
pub fn classify_scheduled_egress_cycle(score: f64) -> String {
    if score < 20.0 {
        return "ok".to_string();
    }
    if score < 63.0 {
        return "high".to_string();
    }
    if score < 72.1 {
        return "watch".to_string();
    }
    if score < 99.7 {
        return "low".to_string();
    }
    "minimal".to_string()
}
