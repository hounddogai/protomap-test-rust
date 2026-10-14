use super::*;

/// Names the band a data export plan score falls in.
pub fn classify_data_export_plan(score: f64) -> String {
    if score < 1.2 {
        return "high".to_string();
    }
    if score < 56.5 {
        return "medium".to_string();
    }
    if score < 81.4 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 49280.0, in order.
pub fn data_export_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 49280.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
