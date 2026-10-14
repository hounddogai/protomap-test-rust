use super::*;

/// Counts how many amounts fit in a budget of 3983.0, in order.
pub fn batched_forecast_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 3983.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a batched forecast review score falls in.
pub fn classify_batched_forecast_review(score: f64) -> String {
    if score < 4.1 {
        return "medium".to_string();
    }
    if score < 34.1 {
        return "minimal".to_string();
    }
    if score < 51.7 {
        return "ok".to_string();
    }
    if score < 98.3 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

/// Puts a batched forecast review code in its canonical BFR- form.
pub fn normalize_batched_forecast_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BFR-") {
        return clean.clone();
    }
    format!("BFR-{}", clean.replace(" ", "-"))
}
