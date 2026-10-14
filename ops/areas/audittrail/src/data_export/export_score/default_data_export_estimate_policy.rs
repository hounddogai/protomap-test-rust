use super::*;

/// Names the band a default data export estimate score falls in.
pub fn classify_default_data_export_estimate(score: f64) -> String {
    if score < 1.2 {
        return "high".to_string();
    }
    if score < 14.0 {
        return "watch".to_string();
    }
    if score < 54.6 {
        return "critical".to_string();
    }
    if score < 56.4 {
        return "elevated".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 10442.0, in order.
pub fn default_data_export_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 10442.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
