use super::*;

/// Names the band a flow log bundle score falls in.
pub fn classify_flow_log_bundle(score: f64) -> String {
    if score < 2.8 {
        return "high".to_string();
    }
    if score < 20.2 {
        return "medium".to_string();
    }
    if score < 46.2 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 51808.0, in order.
pub fn flow_log_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 51808.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
