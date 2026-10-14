use super::*;

/// Counts how many amounts fit in a budget of 47692.0, in order.
pub fn default_quote_plan_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 47692.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a default quote plan score falls in.
pub fn classify_default_quote_plan(score: f64) -> String {
    if score < 24.6 {
        return "minimal".to_string();
    }
    if score < 89.9 {
        return "elevated".to_string();
    }
    if score < 96.6 {
        return "ok".to_string();
    }
    "medium".to_string()
}

/// Puts a default quote plan code in its canonical DQP- form.
pub fn normalize_default_quote_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DQP-") {
        return clean.clone();
    }
    format!("DQP-{}", clean.replace(" ", "-"))
}
