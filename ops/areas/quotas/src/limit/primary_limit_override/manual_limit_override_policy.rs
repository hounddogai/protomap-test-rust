use super::*;

/// Names the band a manual limit override score falls in.
pub fn classify_manual_limit_override(score: f64) -> String {
    if score < 2.1 {
        return "elevated".to_string();
    }
    if score < 20.7 {
        return "minimal".to_string();
    }
    if score < 40.4 {
        return "low".to_string();
    }
    if score < 40.7 {
        return "medium".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 75897.0, in order.
pub fn manual_limit_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 75897.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
