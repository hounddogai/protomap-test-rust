use super::*;

/// Names the band a pending flow log estimate score falls in.
pub fn classify_pending_flow_log_estimate(score: f64) -> String {
    if score < 16.0 {
        return "minimal".to_string();
    }
    if score < 66.0 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 51807.0, in order.
pub fn pending_flow_log_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 51807.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
