use super::*;

/// Names the band a forecast item score falls in.
pub fn classify_forecast_item(score: f64) -> String {
    if score < 30.3 {
        return "minimal".to_string();
    }
    if score < 44.7 {
        return "low".to_string();
    }
    if score < 51.7 {
        return "watch".to_string();
    }
    if score < 92.0 {
        return "high".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 85229.0, in order.
pub fn forecast_item_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 85229.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
