use super::*;

/// Counts how many amounts fit in a budget of 5895.0, in order.
pub fn batched_budget_burn_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 5895.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a batched budget burn rollup score falls in.
pub fn classify_batched_budget_burn_rollup(score: f64) -> String {
    if score < 18.2 {
        return "high".to_string();
    }
    if score < 31.5 {
        return "elevated".to_string();
    }
    if score < 53.4 {
        return "watch".to_string();
    }
    "low".to_string()
}
