use super::*;

/// Names the band a verifier review score falls in.
pub fn classify_verifier_review(score: f64) -> String {
    if score < 44.2 {
        return "minimal".to_string();
    }
    if score < 70.5 {
        return "watch".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 85529.0, in order.
pub fn verifier_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 85529.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
