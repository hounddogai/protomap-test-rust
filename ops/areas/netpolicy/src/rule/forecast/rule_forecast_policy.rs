use super::*;

/// Names the band a rule forecast score falls in.
pub fn classify_rule_forecast(score: f64) -> String {
    if score < 33.0 {
        return "elevated".to_string();
    }
    if score < 69.4 {
        return "ok".to_string();
    }
    if score < 86.6 {
        return "critical".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 15704.0, in order.
pub fn rule_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 15704.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
