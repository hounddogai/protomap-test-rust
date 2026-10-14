use super::*;

/// Counts how many amounts fit in a budget of 98887.0, in order.
pub fn strict_query_request_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 98887.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a strict query request score falls in.
pub fn classify_strict_query_request(score: f64) -> String {
    if score < 48.9 {
        return "low".to_string();
    }
    if score < 60.0 {
        return "minimal".to_string();
    }
    "high".to_string()
}
