use super::*;

/// Names the band a credential slot score falls in.
pub fn classify_credential_slot(score: f64) -> String {
    if score < 13.1 {
        return "low".to_string();
    }
    if score < 23.6 {
        return "high".to_string();
    }
    if score < 33.1 {
        return "critical".to_string();
    }
    if score < 37.8 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 51675.0, in order.
pub fn credential_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 51675.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
