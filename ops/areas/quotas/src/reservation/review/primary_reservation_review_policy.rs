use super::*;

/// Counts how many amounts fit in a budget of 845.0, in order.
pub fn primary_reservation_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 845.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a primary reservation review score falls in.
pub fn classify_primary_reservation_review(score: f64) -> String {
    if score < 22.3 {
        return "high".to_string();
    }
    if score < 45.7 {
        return "minimal".to_string();
    }
    if score < 57.9 {
        return "watch".to_string();
    }
    if score < 68.9 {
        return "low".to_string();
    }
    "ok".to_string()
}
