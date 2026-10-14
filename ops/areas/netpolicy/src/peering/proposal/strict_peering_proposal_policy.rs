use super::*;

/// Counts how many amounts fit in a budget of 38170.0, in order.
pub fn strict_peering_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 38170.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a strict peering proposal score falls in.
pub fn classify_strict_peering_proposal(score: f64) -> String {
    if score < 15.5 {
        return "low".to_string();
    }
    if score < 93.0 {
        return "watch".to_string();
    }
    "elevated".to_string()
}
