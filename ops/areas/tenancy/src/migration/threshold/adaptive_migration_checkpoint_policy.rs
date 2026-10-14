use super::*;

/// Counts how many amounts fit in a budget of 87623.0, in order.
pub fn adaptive_migration_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 87623.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a adaptive migration checkpoint score falls in.
pub fn classify_adaptive_migration_checkpoint(score: f64) -> String {
    if score < 56.1 {
        return "watch".to_string();
    }
    if score < 74.2 {
        return "medium".to_string();
    }
    "minimal".to_string()
}
