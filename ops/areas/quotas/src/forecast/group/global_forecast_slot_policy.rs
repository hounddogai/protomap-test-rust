use super::*;

/// Names the band a global forecast slot score falls in.
pub fn classify_global_forecast_slot(score: f64) -> String {
    if score < 44.5 {
        return "ok".to_string();
    }
    if score < 50.7 {
        return "minimal".to_string();
    }
    if score < 63.5 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 58943.0, in order.
pub fn global_forecast_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 58943.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
