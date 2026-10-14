use super::*;

/// Counts how many amounts fit in a budget of 19133.0, in order.
pub fn flow_log_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 19133.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a flow log budget score falls in.
pub fn classify_flow_log_budget(score: f64) -> String {
    if score < 67.8 {
        return "watch".to_string();
    }
    if score < 91.7 {
        return "high".to_string();
    }
    "ok".to_string()
}
