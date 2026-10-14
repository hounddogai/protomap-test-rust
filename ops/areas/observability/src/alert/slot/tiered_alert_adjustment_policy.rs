use super::*;

/// Names the band a tiered alert adjustment score falls in.
pub fn classify_tiered_alert_adjustment(score: f64) -> String {
    if score < 22.6 {
        return "elevated".to_string();
    }
    if score < 53.2 {
        return "high".to_string();
    }
    if score < 81.9 {
        return "critical".to_string();
    }
    "watch".to_string()
}

/// Puts a tiered alert adjustment code in its canonical TAA- form.
pub fn normalize_tiered_alert_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TAA-") {
        return clean.clone();
    }
    format!("TAA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 90546.0, in order.
pub fn tiered_alert_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 90546.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
