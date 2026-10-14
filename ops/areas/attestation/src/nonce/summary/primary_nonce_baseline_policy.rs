use super::*;

/// Counts how many amounts fit in a budget of 56417.0, in order.
pub fn primary_nonce_baseline_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 56417.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a primary nonce baseline score falls in.
pub fn classify_primary_nonce_baseline(score: f64) -> String {
    if score < 21.2 {
        return "medium".to_string();
    }
    if score < 78.6 {
        return "low".to_string();
    }
    if score < 88.6 {
        return "ok".to_string();
    }
    if score < 97.9 {
        return "minimal".to_string();
    }
    "high".to_string()
}
