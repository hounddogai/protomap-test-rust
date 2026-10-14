use super::*;

/// Names the band a adaptive query adjustment score falls in.
pub fn classify_adaptive_query_adjustment(score: f64) -> String {
    if score < 5.5 {
        return "medium".to_string();
    }
    if score < 16.3 {
        return "minimal".to_string();
    }
    if score < 46.1 {
        return "elevated".to_string();
    }
    if score < 55.3 {
        return "critical".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 17571.0, in order.
pub fn adaptive_query_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 17571.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
